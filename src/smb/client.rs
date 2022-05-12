//! # Client
//!
//! module which exposes the Smb Client

use super::{SmbCredentials, SmbFile, SmbMode, SmbOpenOptions, SmbOptions, SmbStat};
use crate::{utils, SmbDirent};
use crate::{SmbError, SmbResult};

use libc::{self, c_char, c_int, c_uint, c_void};
use smbclient_sys::{SMBCCTX as SmbContext, *};
use std::mem;
use std::panic;
use std::ptr;

// TODO: add logs

/// Smb protocol client
pub struct SmbClient {
    pub(crate) ctx: *mut SmbContext,
    uri: String,
}

impl SmbClient {
    /// Initialize a new `SmbClient` with the provided credentials to connect to the remote smb server
    pub fn new<F>(credentials: SmbCredentials, options: SmbOptions) -> SmbResult<Self>
    where
        F: Fn(&str, &str) -> (String, String, String),
    {
        let mut smbc = SmbClient {
            ctx: ptr::null_mut(),
            uri: Self::build_uri(&credentials),
        };
        let auth_fn = |_: &str, _: &str| -> (String, String, String) {
            (
                credentials.workgroup,
                credentials.username,
                credentials.password,
            )
        };

        unsafe {
            let ctx = utils::result_from_ptr_mut(smbc_new_context())?;
            // set options
            smbc_setOptionUserData(ctx, &auth_fn as *const _ as *mut c_void);
            smbc_setFunctionAuthDataWithContext(ctx, Some(Self::auth_wrapper::<F>));
            Self::setup_options(ctx, options);
            // set ctx
            smbc.ctx = utils::result_from_ptr_mut(smbc_init_context(ctx))?;
        }
        Ok(smbc)
    }

    /// Get netbios name from server
    pub fn get_netbios_name(&self) -> SmbResult<String> {
        unsafe {
            let ptr = utils::result_from_ptr_mut(smbc_getNetbiosName(self.ctx))?;
            utils::char_ptr_to_string(ptr).map_err(|_| SmbError::BadValue)
        }
    }

    /// Set netbios name to server
    pub fn set_netbios_name<S>(&self, name: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let name = utils::str_to_cstring(name)?;
        unsafe { smbc_setNetbiosName(self.ctx, name.into_raw()) }
        Ok(())
    }

    /// Get workgroup name from server
    pub fn get_workgroup(&self) -> SmbResult<String> {
        unsafe {
            let ptr = utils::result_from_ptr_mut(smbc_getWorkgroup(self.ctx))?;
            utils::char_ptr_to_string(ptr).map_err(|_| SmbError::BadValue)
        }
    }

    /// Set workgroup name to server
    pub fn set_workgroup<S>(&self, name: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let name = utils::str_to_cstring(name)?;
        unsafe { smbc_setWorkgroup(self.ctx, name.into_raw()) }
        Ok(())
    }

    /// Get get_user name from server
    pub fn get_user(&self) -> SmbResult<String> {
        unsafe {
            let ptr = utils::result_from_ptr_mut(smbc_getUser(self.ctx))?;
            utils::char_ptr_to_string(ptr).map_err(|_| SmbError::BadValue)
        }
    }

    /// Set user name to server
    pub fn set_user<S>(&self, name: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let name = utils::str_to_cstring(name)?;
        unsafe { smbc_setUser(self.ctx, name.into_raw()) }
        Ok(())
    }

    /// Get timeout from server
    pub fn get_timeout(&self) -> SmbResult<usize> {
        unsafe { Ok(smbc_getTimeout(self.ctx) as usize) }
    }

    /// Set timeout to server
    pub fn set_timeout(&self, timeout: usize) -> SmbResult<()> {
        unsafe { smbc_setTimeout(self.ctx, timeout as c_int) }
        Ok(())
    }

    /// Get smbc version
    pub fn get_version(&self) -> SmbResult<String> {
        unsafe {
            let ptr = smbc_version();
            utils::char_ptr_to_string(ptr).map_err(|_| SmbError::BadValue)
        }
    }

    /// Unlink file at `path`
    pub fn unlink<S>(&self, path: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let path = utils::str_to_cstring(self.uri(path))?;
        unsafe { utils::to_result_with_ioerror((), smbc_unlink(path.as_ptr())) }
    }

    /// Rename file at `orig_url` to `new_url`
    pub fn rename<S>(&self, orig_url: S, new_url: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let orig_url = utils::str_to_cstring(self.uri(orig_url))?;
        let new_url = utils::str_to_cstring(self.uri(new_url))?;
        unsafe {
            utils::to_result_with_ioerror((), smbc_rename(orig_url.as_ptr(), new_url.as_ptr()))
        }
    }

    /// List content of directory at `path`
    pub fn list_dir<S>(&self, path: S) -> SmbResult<Vec<SmbDirent>>
    where
        S: AsRef<str>,
    {
        let path = utils::str_to_cstring(self.uri(path))?;
        unsafe {
            let fd = smbc_opendir(path.as_ptr());
            if fd < 0 {
                error!("failed to open directory: returned a bad file descriptor");
                return Err(SmbError::BadFileDescriptor);
            }
            // seek directory
            if smbc_lseekdir(fd, libc::SEEK_END.into()) < 0 {
                let _ = smbc_closedir(fd);
                error!(
                    "could not seek directory to the end: {}",
                    utils::last_os_error()
                );
                return Err(utils::last_os_error());
            }
            // tell directory
            let dir_size = smbc_telldir(fd);
            if dir_size < 0 {
                let _ = smbc_closedir(fd);
                error!(
                    "could not get directory structure size: {}",
                    utils::last_os_error()
                );
                return Err(utils::last_os_error());
            }
            // Calculate directory size
            let dir_size = dir_size as usize / mem::size_of::<smbc_dirent>();
            trace!("dir_size buffer is {}", dir_size);
            // rewind
            if smbc_lseekdir(fd, libc::SEEK_SET.into()) < 0 {
                let _ = smbc_closedir(fd);
                error!(
                    "could not rewind directory structure: {}",
                    utils::last_os_error()
                );
                return Err(utils::last_os_error());
            }
            // Allocate directory buffer
            let mut buffer: Vec<smbc_dirent> = Vec::with_capacity(dir_size);
            // Get dirents
            let count = smbc_getdents(fd as c_uint, buffer.as_mut_ptr(), i32::MAX);
            if count < 0 {
                let _ = smbc_closedir(fd);
                error!(
                    "could not get directory entries: {}",
                    utils::last_os_error()
                );
                return Err(utils::last_os_error());
            }
            trace!("found {} entries", count);
            let directories = buffer.into_iter().flat_map(SmbDirent::try_from).collect();
            // Close directory
            let _ = smbc_closedir(fd);
            Ok(directories)
        }
    }

    /// Make directory at `p` with provided `mode`
    pub fn mkdir<S>(&self, p: S, mode: SmbMode) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let p = utils::str_to_cstring(self.uri(p))?;
        unsafe { utils::to_result_with_ioerror((), smbc_mkdir(p.as_ptr(), mode.into())) }
    }

    /// Remove directory at `p`
    pub fn rmdir<S>(&self, p: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        let p = utils::str_to_cstring(self.uri(p))?;
        unsafe { utils::to_result_with_ioerror((), smbc_rmdir(p.as_ptr())) }
    }

    /// Stat file at `p` and return its metadata
    pub fn stat<S>(&self, p: S) -> SmbResult<SmbStat>
    where
        S: AsRef<str>,
    {
        trace!("Stating file at {}", p.as_ref());
        let p = utils::str_to_cstring(self.uri(p))?;
        unsafe {
            let mut st: libc::stat = mem::zeroed();
            if smbc_stat(p.as_ptr(), &mut st) < 0 {
                error!("failed to stat file: {}", utils::last_os_error());
                Err(utils::last_os_error())
            } else {
                Ok(SmbStat::from(st))
            }
        }
    }

    /// Change file mode for file at `p`
    pub fn chmod<S>(&self, p: S, mode: SmbMode) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        trace!("changing mode for {} with {:?}", p.as_ref(), mode);
        let p = utils::str_to_cstring(self.uri(p))?;
        unsafe { utils::to_result_with_ioerror((), smbc_chmod(p.as_ptr(), mode.into())) }
    }

    /// Print file at `p` using the `print_queue`
    pub fn print<S>(&self, p: S, print_queue: S) -> SmbResult<()>
    where
        S: AsRef<str>,
    {
        trace!("printing {} to {} queue", p.as_ref(), print_queue.as_ref());
        let p = utils::str_to_cstring(self.uri(p))?;
        let print_queue = utils::str_to_cstring(self.uri(print_queue))?;
        unsafe {
            utils::to_result_with_ioerror((), smbc_print_file(p.as_ptr(), print_queue.as_ptr()))
        }
    }

    // -- internal private

    /// Build connection uri
    fn build_uri(SmbCredentials { server, share, .. }: &SmbCredentials) -> String {
        format!(
            "{}{}{}",
            server,
            match share.starts_with('/') {
                true => "",
                false => "/",
            },
            share
        )
    }

    /// Get file uri
    fn uri<S>(&self, p: S) -> String
    where
        S: AsRef<str>,
    {
        format!("{}{}", self.uri, p.as_ref())
    }

    /// Callback to get file descriptor
    pub(crate) fn get_fn<T>(
        &self,
        get_func: unsafe extern "C" fn(*mut SMBCCTX) -> Option<T>,
    ) -> std::io::Result<T> {
        unsafe {
            get_func(self.ctx).ok_or_else(|| std::io::Error::from_raw_os_error(libc::EINVAL as i32))
        }
    }

    /// Setup options in the context
    unsafe fn setup_options(ctx: *mut SMBCCTX, options: SmbOptions) {
        smbc_setOptionBrowseMaxLmbCount(ctx, options.browser_max_lmb_count);
        smbc_setOptionCaseSensitive(ctx, options.case_sensitive as i32);
        smbc_setOptionDebugToStderr(ctx, 0);
        smbc_setOptionFallbackAfterKerberos(ctx, options.fallback_after_kerberos as i32);
        smbc_setOptionNoAutoAnonymousLogin(ctx, options.no_auto_anonymous_login as i32);
        smbc_setOptionOneSharePerServer(ctx, options.one_share_per_server as i32);
        smbc_setOptionOpenShareMode(ctx, options.open_share_mode.into());
        smbc_setOptionSmbEncryptionLevel(ctx, options.encryption_level.into());
        smbc_setOptionUrlEncodeReaddirEntries(ctx, options.url_encode_readdir_entries as i32);
        smbc_setOptionUseCCache(ctx, options.use_ccache as i32);
        smbc_setOptionUseKerberos(ctx, options.use_kerberos as i32);
    }

    /// Auth wrapper passed to `SMBCCTX` to authenticate requests to SMB servers.
    extern "C" fn auth_wrapper<F>(
        ctx: *mut SMBCCTX,
        srv: *const c_char,
        shr: *const c_char,
        wg: *mut c_char,
        wglen: c_int,
        un: *mut c_char,
        unlen: c_int,
        pw: *mut c_char,
        pwlen: c_int,
    ) where
        F: Fn(&str, &str) -> (String, String, String),
    {
        unsafe {
            let srv = utils::cstr(srv);
            let shr = utils::cstr(shr);
            trace!("authenticating on {}\\{}", &srv, &shr);

            let auth: &F = &*(smbc_getOptionUserData(ctx) as *const c_void as *const F);
            let auth = panic::AssertUnwindSafe(auth);
            let (workgroup, username, password) = panic::catch_unwind(|| {
                trace!("auth with {:?}\\{:?}", srv, shr);
                auth(&srv, &shr)
            })
            .unwrap();
            trace!("cred: {}\\{} {}", &workgroup, &username, &password);
            utils::write_to_cstr(wg as *mut u8, wglen as usize, &workgroup);
            utils::write_to_cstr(un as *mut u8, unlen as usize, &username);
            utils::write_to_cstr(pw as *mut u8, pwlen as usize, &password);
        }
    }
}

impl<'a> SmbClient {
    /// Open a file at `P` with provided options
    pub fn open_with<P: AsRef<str>>(
        &'a self,
        path: P,
        options: SmbOpenOptions,
    ) -> SmbResult<SmbFile<'a>> {
        trace!("opening {} with {:?}", path.as_ref(), options);
        let open_fn = self.get_fn(smbc_getFunctionOpen)?;
        let path = utils::str_to_cstring(self.uri(path))?;
        let fd = utils::result_from_ptr_mut(open_fn(
            self.ctx,
            path.as_ptr(),
            options.to_flags(),
            options.mode,
        ))?;
        if (fd as i64) < 0 {
            error!("got a negative file descriptor");
            Err(SmbError::BadFileDescriptor)
        } else {
            trace!("opened file with file descriptor {:?}", fd);
            Ok(SmbFile::new(self, fd))
        }
    }
}

// -- destructor
impl Drop for SmbClient {
    fn drop(&mut self) {
        trace!("closing smbclient");
        unsafe {
            smbc_free_context(self.ctx, 1_i32);
        }
        trace!("smbclient context freed");
    }
}

#[cfg(test)]
#[cfg(feature = "with-containers")]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_initialize_client() {
        todo!();
    }

    #[test]
    fn should_get_netbios() {
        todo!();
    }

    #[test]
    fn should_set_netbios() {
        todo!();
    }

    #[test]
    fn should_get_workgroup() {
        todo!();
    }

    #[test]
    fn should_set_workgroup() {
        todo!();
    }

    #[test]
    fn should_get_user() {
        todo!();
    }

    #[test]
    fn should_set_user() {
        todo!();
    }

    #[test]
    fn should_get_timeout() {
        todo!();
    }

    #[test]
    fn should_set_timeout() {
        todo!();
    }

    #[test]
    fn should_get_version() {
        todo!();
    }

    #[test]
    fn should_unlink() {
        todo!();
    }

    #[test]
    fn should_rename() {
        todo!();
    }

    #[test]
    fn should_list_dir() {
        todo!();
    }

    #[test]
    fn should_mkdir() {
        todo!();
    }

    #[test]
    fn should_rmdir() {
        todo!();
    }

    #[test]
    fn should_stat() {
        todo!();
    }

    #[test]
    fn should_chmod() {
        todo!();
    }

    #[test]
    fn should_build_uri() {
        todo!();
    }

    #[test]
    fn should_read_file() {
        todo!();
    }

    #[test]
    fn should_write_file() {
        todo!();
    }

    #[test]
    fn should_append_to_file() {
        todo!();
    }
}
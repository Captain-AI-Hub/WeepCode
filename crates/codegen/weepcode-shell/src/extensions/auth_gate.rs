use agent_client_protocol as acp;

use crate::auth::{AuthManager, WeepcodeAuth};

/// Require WeepCode auth from a sync context, accepting tokens in the client-side buffer window.
pub(crate) fn require_weepcode_auth(
    auth_manager: &AuthManager,
    missing_message: &'static str,
    non_weepcode_message: &'static str,
) -> Result<WeepcodeAuth, acp::Error> {
    let auth = auth_manager
        .current_or_expired()
        .ok_or_else(|| acp::Error::auth_required().data(missing_message))?;
    if !auth.is_weepcode_auth() {
        return Err(acp::Error::auth_required().data(non_weepcode_message));
    }
    Ok(auth)
}

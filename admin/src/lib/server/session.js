// Admin session handling: the backend JWT lives only in an httpOnly cookie, so browser
// JavaScript (and any injected script) can never read it.

export const SESSION_COOKIE = 'admin_session';
const SESSION_MAX_AGE = 60 * 60 * 12; // matches the backend token lifetime (12 h)

export const backendUrl = () => process.env.BACKEND_INTERNAL_URL || 'http://backend:8000';

export function setSessionCookie(cookies, token, url) {
  cookies.set(SESSION_COOKIE, token, {
    path: '/',
    httpOnly: true,
    sameSite: 'strict',
    secure: url.protocol === 'https:',
    maxAge: SESSION_MAX_AGE
  });
}

export function clearSessionCookie(cookies) {
  cookies.delete(SESSION_COOKIE, { path: '/' });
}

/** Asks the backend who the token belongs to; null if the session is invalid or expired. */
export async function fetchSessionAdmin(token) {
  if (!token) return null;
  try {
    const res = await fetch(`${backendUrl()}/api/v1/admin/auth/me`, {
      headers: { Authorization: `Bearer ${token}` }
    });
    if (res.ok) return await res.json();
    // A valid session that still has to replace its initial password
    if (res.status === 403) {
      const body = await res.json().catch(() => ({}));
      if (body.code === 'password_change_required') return { is_default: true };
    }
  } catch (e) {
    console.error('Admin session check failed:', e);
  }
  return null;
}

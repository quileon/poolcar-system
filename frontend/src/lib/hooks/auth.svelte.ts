import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

class AuthState {
	logout() {
		if (browser) {
			goto(resolve("/login"));
		}
	}
}

export const authState = new AuthState();

/**
 * Authenticated fetch wrapper.
 * - Cookies are automatically sent with requests.
 * - On 401 responses, automatically logs out and redirects to login.
 */
export async function authFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
	const response = await fetch(input, {
		...init
	});

	if (response.status === 401) {
		authState.logout();
		throw new Error("Unauthorized");
	}

	return response;
}

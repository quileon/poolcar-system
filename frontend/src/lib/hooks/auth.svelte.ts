import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

class AuthState {
	#token: string | null = $state(browser ? localStorage.getItem("authToken") : null);

	get isAuthenticated(): boolean {
		return this.#token !== null;
	}

	get token(): string | null {
		return this.#token;
	}

	setToken(token: string) {
		this.#token = token;
		if (browser) {
			localStorage.setItem("authToken", token);
		}
	}

	clearToken() {
		this.#token = null;
		if (browser) {
			localStorage.removeItem("authToken");
		}
	}

	guard() {
		if (browser && !this.isAuthenticated) {
			this.clearToken();
			goto(resolve("/login"));
		}
	}

	logout() {
		this.clearToken();
		if (browser) {
			goto(resolve("/login"));
		}
	}
}

export const authState = new AuthState();

/**
 * Authenticated fetch wrapper.
 * - Injects `Authorization: Bearer <token>` header into request.
 * - On 401 responses, automatically logs out and redirects to login.
 */
export async function authFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
	const headers = new Headers(init?.headers);

	const token = authState.token;
	if (token) {
		headers.set("Authorization", `Bearer ${token}`);
	}

	const response = await fetch(input, { ...init, headers });

	if (response.status === 401) {
		authState.logout();
		throw new Error("Unauthorized");
	}

	return response;
}

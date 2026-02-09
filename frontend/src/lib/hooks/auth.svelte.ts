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

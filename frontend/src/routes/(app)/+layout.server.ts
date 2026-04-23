import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { config } from "$lib/config";
import type { LoginResponse } from "$lib/bindings/LoginResponse";
import type { SuccessDataResponse } from "$lib/bindings/SuccessDataResponse";

export const load: LayoutServerLoad = async ({ cookies, fetch }) => {
	const token = cookies.get("auth_token");
	if (!token) {
		throw redirect(302, "/login");
	}

	let verifyToken;
	let verifyResponse: SuccessDataResponse;

	try {
		verifyToken = await fetch(`${config.apiBaseUrl}/auth/verify`, {
			headers: {
				Cookie: `auth_token=${token}`
			}
		});
		verifyResponse = await verifyToken.json();
	} catch (error) {
		console.error("Fetch error verifying token:", error);
		throw redirect(302, "/login");
	}

	if (!verifyToken.ok || verifyResponse.status !== "success" || !verifyResponse.data) {
		console.error("Verification failed:", { ok: verifyToken.ok, response: verifyResponse });
		throw redirect(302, "/login");
	}

	const userData = verifyResponse.data as LoginResponse;

	return { userData };
};

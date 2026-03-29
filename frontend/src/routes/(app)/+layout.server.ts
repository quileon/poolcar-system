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

	const verifyToken = await fetch(`${config.apiBaseUrl}/auth/verify`);
	const verifyResponse: SuccessDataResponse = await verifyToken.json();

	if (!verifyToken.ok || verifyResponse.status !== "success" || !verifyResponse.data) {
		throw redirect(302, "/login");
	}

	const userData = verifyResponse.data as LoginResponse;

	return { userData };
};

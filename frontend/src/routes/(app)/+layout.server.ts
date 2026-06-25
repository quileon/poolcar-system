import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { config } from "$lib/config";
import type { LoginResponse } from "$lib/bindings/LoginResponse";
import type { SuccessDataResponse } from "$lib/bindings/SuccessDataResponse";

export const load: LayoutServerLoad = async ({ cookies, fetch, url }) => {
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

	if (userData.role === "Security") {
		cookies.delete("auth_token", { path: "/" });
		throw redirect(302, "/login");
	}

	const managementPaths = [
		"/cars",
		"/car-types",
		"/car-status",
		"/trackers",
		"/contacts",
		"/contact-types",
		"/activities",
		"/activity-types",
		"/users",
		"/user-roles"
	];

	if (userData.role !== "Admin") {
		const isManagementPath = managementPaths.some(
			(path) => url.pathname === path || url.pathname.startsWith(path + "/")
		);
		if (isManagementPath) {
			throw redirect(302, "/");
		}
	}

	return { userData };
};

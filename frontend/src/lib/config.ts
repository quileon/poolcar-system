import { env } from "$env/dynamic/public";

if (!env.PUBLIC_API_BASE_URL) {
	throw new Error("PUBLIC_API_BASE_URL environment variable is not defined");
}

if (!env.PUBLIC_WS_BASE_URL) {
	throw new Error("PUBLIC_WS_BASE_URL environment variable is not defined");
}

export const config = {
	apiBaseUrl: env.PUBLIC_API_BASE_URL,
	wsBaseUrl: env.PUBLIC_WS_BASE_URL
} as const;

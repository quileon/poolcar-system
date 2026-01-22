import { PUBLIC_API_BASE_URL } from "$env/static/public";

if (!PUBLIC_API_BASE_URL) {
	throw new Error("PUBLIC_API_BASE_URL environment variable is not defined");
}

export const config = {
	apiBaseUrl: PUBLIC_API_BASE_URL
} as const;

import { createQuery } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import type { GoogleMapResponse } from "$lib/bindings/GoogleMapResponse";
import type { SuccessDataResponse } from "$lib/bindings/SuccessDataResponse";

export function useSearchPlacesQuery(getName: () => string | null) {
	return createQuery<GoogleMapResponse>(() => {
		const name = getName();
		const searchParams = new URLSearchParams();
		if (name) {
			searchParams.set("name", name);
		}

		return {
			queryKey: ["search-places", name],
			queryFn: async () => {
				const response = await authFetch(`${config.apiBaseUrl}/search?${searchParams.toString()}`);
				if (!response.ok) throw new Error("Failed to search places");
				const successResponse = (await response.json()) as SuccessDataResponse;
				return successResponse.data as GoogleMapResponse;
			},
			enabled: !!name
		};
	});
}

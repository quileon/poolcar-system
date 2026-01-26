import { createQuery } from "@tanstack/svelte-query";
import { config } from "$lib/config";
import type { GetTrackerResponse } from "$lib/bindings/GetTrackerResponse";
import type { GetCarTypesResponse } from "$lib/bindings/GetCarTypesResponse";
import type { GetCarsResponse } from "$lib/bindings/GetCarsResponse";

export function useTrackersQuery() {
	return createQuery<GetTrackerResponse>(() => ({
		queryKey: ["trackers"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/trackers`);
			if (!response.ok) throw new Error("Failed to fetch trackers");
			return response.json();
		}
	}));
}

export function useCarTypesQuery() {
	return createQuery<GetCarTypesResponse>(() => ({
		queryKey: ["car-types"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types`);
			if (!response.ok) throw new Error("Failed to fetch car types");
			return response.json();
		}
	}));
}

export function useCarsQuery() {
	return createQuery<GetCarsResponse>(() => ({
		queryKey: ["cars"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars`);
			if (!response.ok) throw new Error("Failed to fetch cars");
			return response.json();
		}
	}));
}

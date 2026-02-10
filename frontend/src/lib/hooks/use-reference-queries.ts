import { createQuery } from "@tanstack/svelte-query";
import { config } from "$lib/config";
import { authFetch } from "$lib/hooks/auth.svelte";
import type { GetTrackerResponse } from "$lib/bindings/GetTrackerResponse";
import type { GetMqttPayloadHistory } from "$lib/bindings/GetMqttPayloadHistory";
import type { GetCarTypesResponse } from "$lib/bindings/GetCarTypesResponse";
import type { GetCarsResponse } from "$lib/bindings/GetCarsResponse";

export function useTrackersQuery() {
	return createQuery<GetTrackerResponse>(() => ({
		queryKey: ["trackers"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/trackers`);
			if (!response.ok) throw new Error("Failed to fetch trackers");
			return response.json();
		}
	}));
}

export function useCarTypesQuery() {
	return createQuery<GetCarTypesResponse>(() => ({
		queryKey: ["car-types"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/types`);
			if (!response.ok) throw new Error("Failed to fetch car types");
			return response.json();
		}
	}));
}

export function useCarsQuery() {
	return createQuery<GetCarsResponse>(() => ({
		queryKey: ["cars"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/cars`);
			if (!response.ok) throw new Error("Failed to fetch cars");
			return response.json();
		}
	}));
}

export function useMqttPayloadHistoriesQuery() {
	return createQuery<GetMqttPayloadHistory>(() => ({
		queryKey: ["mqtt-payload-histories"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/`);
			if (!response.ok) throw new Error("Failed to fetch MQTT payload histories");
			return response.json();
		}
	}));
}

import { createQuery } from "@tanstack/svelte-query";
import { config } from "$lib/config";
import { authFetch } from "$lib/hooks/auth.svelte";
import type { CarWithTracker } from "$lib/bindings/CarWithTracker";
import type { CarTypeWithCount } from "$lib/bindings/CarTypeWithCount";
import type { TrackerWithDetails } from "$lib/bindings/TrackerWithDetails";

export function useTrackerQuery(getTrackerId: () => number) {
	return createQuery<TrackerWithDetails>(() => ({
		queryKey: ["tracker", getTrackerId()],
		queryFn: async () => {
			const trackerId = getTrackerId();
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}`);
			if (!response.ok) throw new Error("Failed to fetch tracker");
			return response.json();
		}
	}));
}

export function useCarTypeQuery(getCarTypeId: () => number) {
	return createQuery<CarTypeWithCount>(() => ({
		queryKey: ["car-type", getCarTypeId()],
		queryFn: async () => {
			const carTypeId = getCarTypeId();
			const response = await authFetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`);
			if (!response.ok) throw new Error("Failed to fetch car type");
			return response.json();
		}
	}));
}

export function useCarQuery(getCarId: () => number) {
	return createQuery<CarWithTracker>(() => ({
		queryKey: ["car", getCarId()],
		queryFn: async () => {
			const carId = getCarId();
			const response = await authFetch(`${config.apiBaseUrl}/cars/${carId}`);
			if (!response.ok) throw new Error("Failed to fetch car");
			return response.json();
		}
	}));
}

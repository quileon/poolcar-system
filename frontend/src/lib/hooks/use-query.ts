import { createQuery } from "@tanstack/svelte-query";
import { config } from "$lib/config";
import type { CarWithTracker } from "$lib/bindings/CarWithTracker";
import type { CarTypeWithCount } from "$lib/bindings/CarTypeWithCount";
import type { TrackerWithDetails } from "$lib/bindings/TrackerWithDetails";

export function useTrackerQuery(trackerId: number) {
	return createQuery<TrackerWithDetails>(() => ({
		queryKey: ["tracker", trackerId],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/trackers/${trackerId}`);
			if (!response.ok) throw new Error("Failed to fetch tracker");
			return response.json();
		}
	}));
}

export function useCarTypeQuery(carTypeId: number) {
	return createQuery<CarTypeWithCount>(() => ({
		queryKey: ["car-type", carTypeId],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`);
			if (!response.ok) throw new Error("Failed to fetch car type");
			return response.json();
		}
	}));
}

export function useCarQuery(carId: number) {
	return createQuery<CarWithTracker>(() => ({
		queryKey: ["car", carId],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/${carId}`);
			if (!response.ok) throw new Error("Failed to fetch car");
			return response.json();
		}
	}));
}

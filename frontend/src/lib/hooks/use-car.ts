import type { CarWithTracker } from "$lib/bindings/CarWithTracker";
import type { GetCarsResponse } from "$lib/bindings/GetCarsResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

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

export function useCreateCarMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carName: string;
			policeNumber: string;
			carTypeId: number;
			trackerId: number | null;
			active: boolean;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/cars`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.carName,
					police_number: data.policeNumber,
					car_type_id: data.carTypeId,
					tracker_id: data.trackerId,
					active: data.active
				})
			});
			if (!response.ok) throw new Error("Failed to create car");
			return response.json();
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["cars", "car-types", "trackers"] });
			goto(resolve("/cars"));
		}
	}));
}

export function useEditCarMutation(getCarId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carName: string;
			policeNumber: string;
			carTypeId: number;
			trackerId: number | null;
			active: boolean;
		}) => {
			const carId = getCarId();
			const response = await authFetch(`${config.apiBaseUrl}/cars/${carId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.carName,
					police_number: data.policeNumber,
					car_type_id: data.carTypeId,
					tracker_id: data.trackerId,
					active: data.active
				})
			});
			if (!response.ok) throw new Error("Failed to modify car");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/cars"));
			await queryClient.invalidateQueries({ queryKey: ["cars"] });
		}
	}));
}

export function useDeleteCarMutation(getCarId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const carId = getCarId();
			const response = await authFetch(`${config.apiBaseUrl}/cars/${carId}`, {
				method: "DELETE"
			});
			if (!response.ok) {
				throw new Error("Failed to delete car");
			}
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/cars"));
			await queryClient.invalidateQueries({ queryKey: ["cars"] });
		}
	}));
}

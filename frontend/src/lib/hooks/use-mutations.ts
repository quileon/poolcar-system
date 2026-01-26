import { createMutation, useQueryClient } from "@tanstack/svelte-query";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { config } from "$lib/config";

export function useEditTrackerMutation(trackerId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await fetch(`${config.apiBaseUrl}/trackers/${trackerId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify tracker");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/trackers"));
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
		}
	}));
}

export function useDeleteTrackerMutation(trackerId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/trackers/${trackerId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete tracker");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/trackers"));
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
		}
	}));
}

export function useEditCarTypeMutation(carTypeId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify car type");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/car-types"));
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
		}
	}));
}

export function useDeleteCarTypeMutation(carTypeId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete car type");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/car-types"));
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
		}
	}));
}

export function useEditCarMutation(carId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carName: string;
			policeNumber: string;
			carTypeId: number;
			trackerId: number | null;
			active: boolean;
		}) => {
			const response = await fetch(`${config.apiBaseUrl}/cars/${carId}`, {
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

export function useDeleteCarMutation(carId: number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/${carId}`, {
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

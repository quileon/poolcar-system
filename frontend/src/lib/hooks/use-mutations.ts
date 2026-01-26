import { createMutation, useQueryClient } from "@tanstack/svelte-query";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { config } from "$lib/config";

export function useCreateTrackerMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await fetch(`${config.apiBaseUrl}/trackers`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({ name: data.name })
			});
			if (!response.ok) {
				throw new Error("Failed to create tracker");
			}
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
			goto(resolve("/trackers"));
		}
	}));
}

export function useEditTrackerMutation(getTrackerId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const trackerId = getTrackerId();
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

export function useDeleteTrackerMutation(getTrackerId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const trackerId = getTrackerId();
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

export function useCreateCarTypeMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({ name: data.name })
			});
			if (!response.ok) {
				throw new Error("Failed to create car type");
			}
			return response.json();
		},

		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
			goto(resolve("/car-types"));
		}
	}));
}

export function useEditCarTypeMutation(getCarTypeId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const carTypeId = getCarTypeId();
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

export function useDeleteCarTypeMutation(getCarTypeId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const carTypeId = getCarTypeId();
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
			const response = await fetch(`${config.apiBaseUrl}/cars`, {
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

export function useDeleteCarMutation(getCarId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const carId = getCarId();
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

import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import type { GetCarStatusesResponse } from "$lib/bindings/GetCarStatusesResponse";
import type { CarStatusDetails } from "$lib/bindings/CarStatusDetails";

export function useCarStatusesQuery(getStatus: () => string | null) {
	return createQuery<GetCarStatusesResponse>(() => {
		const status = getStatus();
		const searchParams = new URLSearchParams();
		if (status) {
			searchParams.set("status", status);
		}

		return {
			queryKey: ["car-statuses", status],
			queryFn: async () => {
				const response = await authFetch(
					`${config.apiBaseUrl}/cars/status?${searchParams.toString()}`
				);
				if (!response.ok) throw new Error("Failed to fetch car statuses");
				return response.json();
			}
		};
	});
}

export function useCarStatusQuery(getCarStatusId: () => number) {
	return createQuery<CarStatusDetails>(() => ({
		queryKey: ["car-status", getCarStatusId()],
		queryFn: async () => {
			const carStatusId = getCarStatusId();
			const response = await authFetch(`${config.apiBaseUrl}/cars/status/${carStatusId}`);
			if (!response.ok) throw new Error("Failed to fetch car status");
			return response.json();
		}
	}));
}

export function useCreateCarStatusMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number;
			statusType: "DEPARTURE" | "RETURN";
			gasLevel: number;
			kilometres: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/status`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					status_type: data.statusType,
					gas_level: data.gasLevel,
					kilometres: data.kilometres
				})
			});
			if (!response.ok) throw new Error("Failed to create car status");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-statuses"] });
			await goto(resolve("/car-status"));
		}
	}));
}

export function useEditCarStatusMutation(getCarStatusId: () => number) {
	const queryClient = useQueryClient();
	const carStatusId = getCarStatusId();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number;
			statusType: "DEPARTURE" | "RETURN";
			gasLevel: number;
			kilometres: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/status/${carStatusId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					status_type: data.statusType,
					gas_level: data.gasLevel,
					kilometres: data.kilometres
				})
			});
			if (!response.ok) throw new Error("Failed to modify car status");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-statuses"] });
			await queryClient.invalidateQueries({ queryKey: ["car-status", carStatusId] });
			await goto(resolve("/car-status"));
		}
	}));
}

export function useDeleteCarStatusMutation(getCarStatusId: () => number) {
	const queryClient = useQueryClient();
	const carStatusId = getCarStatusId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/status/${carStatusId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete car status");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-statuses"] });
			await queryClient.invalidateQueries({ queryKey: ["car-status", carStatusId] });
			await goto(resolve("/car-status"));
		}
	}));
}

export function useRestoreCarStatusMutation(getCarStatusId: () => number) {
	const queryClient = useQueryClient();
	const carStatusId = getCarStatusId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/status/${carStatusId}/restore`, {
				method: "PUT"
			});
			if (!response.ok) throw new Error("Failed to restore car status");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-statuses"] });
			await queryClient.invalidateQueries({ queryKey: ["car-status", carStatusId] });
			await goto(resolve("/car-status"));
		}
	}));
}

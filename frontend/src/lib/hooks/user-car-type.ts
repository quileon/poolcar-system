import type { CarTypeWithCount } from "$lib/bindings/CarTypeWithCount";
import type { GetCarTypesResponse } from "$lib/bindings/GetCarTypesResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

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

export function useCreateCarTypeMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/cars/types`, {
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
			const response = await authFetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
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
			const response = await authFetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
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

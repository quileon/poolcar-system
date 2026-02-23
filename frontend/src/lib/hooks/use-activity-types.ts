import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import type { GetActivityTypesResponse } from "$lib/bindings/GetActivityTypesResponse";
import type { ActivityTypeDetails } from "$lib/bindings/ActivityTypeDetails";

export function useActivityTypesQuery() {
	return createQuery<GetActivityTypesResponse>(() => ({
		queryKey: ["activity-types"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/types`);
			if (!response.ok) throw new Error("Failed to fetch activity types");
			return response.json();
		}
	}));
}

export function useActivityTypeQuery(getActivityTypeId: () => number) {
	const activityTypeId = getActivityTypeId();

	return createQuery<ActivityTypeDetails>(() => ({
		queryKey: ["activity-type", activityTypeId],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/types/${activityTypeId}`);
			if (!response.ok) throw new Error("Failed to fetch activity type");
			return response.json();
		}
	}));
}

export function useCreateActivityTypeMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/types`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to create activity type");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activity-types"] });
			await goto(resolve("/activity-types"));
		}
	}));
}

export function useEditActivityTypeMutation(getActivityTypeId: () => number) {
	const queryClient = useQueryClient();
	const activityTypeId = getActivityTypeId();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/types/${activityTypeId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify activity type");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activity-types"] });
			await queryClient.invalidateQueries({ queryKey: ["activity-type", activityTypeId] });
			await goto(resolve("/activity-types"));
		}
	}));
}

export function useDeleteActivityTypeMutation(getActivityTypeId: () => number) {
	const queryClient = useQueryClient();
	const activityTypeId = getActivityTypeId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/types/${activityTypeId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete activity type");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activity-types"] });
			await queryClient.invalidateQueries({ queryKey: ["activity-type", activityTypeId] });
			await goto(resolve("/activity-types"));
		}
	}));
}

export function useRestoreActivityTypeMutation(getActivityTypeId: () => number) {
	const queryClient = useQueryClient();
	const activityTypeId = getActivityTypeId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(
				`${config.apiBaseUrl}/activities/types/${activityTypeId}/restore`,
				{
					method: "PUT"
				}
			);
			if (!response.ok) throw new Error("Failed to restore activity type");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activity-types"] });
			await queryClient.invalidateQueries({ queryKey: ["activity-type", activityTypeId] });
			await goto(resolve("/activity-types"));
		}
	}));
}

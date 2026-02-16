import type { ActivityWithCount } from "$lib/bindings/ActivityWithCount";
import type { GetActivitiesResponse } from "$lib/bindings/GetActivitiesResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useActivitiesQuery() {
	return createQuery<GetActivitiesResponse[]>(() => ({
		queryKey: ["activities"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities`);
			if (!response.ok) throw new Error("Failed to fetch activities");
			return response.json();
		}
	}));
}

export function useActivityQuery(getActivityId: () => number) {
	return createQuery<ActivityWithCount>(() => ({
		queryKey: ["activity", getActivityId()],
		queryFn: async () => {
			const activityId = getActivityId();
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`);
			if (!response.ok) throw new Error("Failed to fetch activity");
			return response.json();
		}
	}));
}

export function useCreateActivityMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/activities`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to create activity");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/activity"));
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
		}
	}));
}

export function useEditActivityMutation(getActivityId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const activityId = getActivityId();
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify activity");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/activity"));
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
		}
	}));
}

export function useDeleteActivityMutation(getActivityId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const activityId = getActivityId();
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete activity");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/activity"));
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
		}
	}));
}

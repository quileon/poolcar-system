import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import type { GetActivitiesResponse } from "$lib/bindings/GetActivitiesResponse";
import type { ActivityDetails } from "$lib/bindings/ActivityDetails";

export function useActivitiesQuery(getStatus: () => string | null) {
	return createQuery<GetActivitiesResponse>(() => {
		const status = getStatus();
		const searchParams = new URLSearchParams();
		if (status) {
			searchParams.set("status", status);
		}

		return {
			queryKey: ["activities", status],
			queryFn: async () => {
				const response = await authFetch(
					`${config.apiBaseUrl}/activities?${searchParams.toString()}`
				);
				if (!response.ok) throw new Error("Failed to fetch activities");
				return response.json();
			}
		};
	});
}

export function useActivityQuery(getActivityId: () => number) {
	const activityId = getActivityId();

	return createQuery<ActivityDetails>(() => ({
		queryKey: ["activity", activityId],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`);
			if (!response.ok) throw new Error("Failed to fetch activity");
			return response.json();
		}
	}));
}

export function useCreateActivityMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number | null;
			contactId: number;
			activityTypeId: number;
			trackerId: number | null;
			startedAt: string;
			finishedAt: string | null;
			finishedLatitude: number | null;
			finishedLongitude: number | null;
			description: string | null;
		}) => {
			console.log(data);
			const response = await authFetch(`${config.apiBaseUrl}/activities`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					contact_id: data.contactId,
					activity_type_id: data.activityTypeId,
					tracker_id: data.trackerId,
					started_at: data.startedAt,
					finished_at: data.finishedAt,
					finished_latitude: data.finishedLatitude,
					finished_longitude: data.finishedLongitude,
					description: data.description
				})
			});
			if (!response.ok) throw new Error("Failed to create activity");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
			await goto(resolve("/activities"));
		}
	}));
}

export function useEditActivityMutation(getActivityId: () => number) {
	const queryClient = useQueryClient();
	const activityId = getActivityId();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number | null;
			contactId: number;
			activityTypeId: number;
			trackerId: number | null;
			startedAt: string;
			finishedAt: string | null;
			finishedLatitude: number | null;
			finishedLongitude: number | null;
			description: string | null;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					contact_id: data.contactId,
					activity_type_id: data.activityTypeId,
					tracker_id: data.trackerId,
					started_at: data.startedAt,
					finished_at: data.finishedAt,
					finished_latitude: data.finishedLatitude,
					finished_longitude: data.finishedLongitude,
					description: data.description
				})
			});
			if (!response.ok) throw new Error("Failed to modify activity");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
			await queryClient.invalidateQueries({ queryKey: ["activity", activityId] });
			await goto(resolve("/activities"));
		}
	}));
}

export function useDeleteActivityMutation(getActivityId: () => number) {
	const queryClient = useQueryClient();
	const activityId = getActivityId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete activity");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
			await queryClient.invalidateQueries({ queryKey: ["activity", activityId] });
			await goto(resolve("/activities"));
		}
	}));
}

export function useRestoreActivityMutation(getActivityId: () => number) {
	const queryClient = useQueryClient();
	const activityId = getActivityId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/activities/${activityId}/restore`, {
				method: "PUT"
			});
			if (!response.ok) throw new Error("Failed to restore activity");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["activities"] });
			await queryClient.invalidateQueries({ queryKey: ["activity", activityId] });
			await goto(resolve("/activities"));
		}
	}));
}

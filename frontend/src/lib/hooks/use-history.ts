import type { GetHistoriesResponse } from "$lib/bindings/GetHistoriesResponse";
import type { HistoryWithDetails } from "$lib/bindings/HistoryWithDetails";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useHistoriesQuery() {
	return createQuery<GetHistoriesResponse[]>(() => ({
		queryKey: ["histories"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/histories`);
			if (!response.ok) throw new Error("Failed to fetch histories");
			return response.json();
		}
	}));
}

export function useHistoryQuery(getHistoryId: () => number) {
	return createQuery<HistoryWithDetails>(() => ({
		queryKey: ["history", getHistoryId()],
		queryFn: async () => {
			const historyId = getHistoryId();
			const response = await authFetch(`${config.apiBaseUrl}/histories/${historyId}`);
			if (!response.ok) throw new Error("Failed to fetch history");
			return response.json();
		}
	}));
}

export function useCreateHistoryMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number;
			contactId: number;
			activityId: number;
			trackerId: number;
			startedAt: string;
			finishedAt: string;
			finishedLatitude: number;
			finishedLongitude: number;
			description: string;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/histories`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					contact_id: data.contactId,
					activity_id: data.activityId,
					tracker_id: data.trackerId,
					started_at: data.startedAt,
					finished_at: data.finishedAt,
					finished_latitude: data.finishedLatitude,
					finished_longitude: data.finishedLongitude,
					description: data.description
				})
			});
			if (!response.ok) throw new Error("Failed to create history");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/history"));
			await queryClient.invalidateQueries({ queryKey: ["histories"] });
		}
	}));
}

export function useEditHistoryMutation(getHistoryId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			carId: number;
			contactId: number;
			activityId: number;
			trackerId: number;
			startedAt: string;
			finishedAt: string;
			finishedLatitude: number;
			finishedLongitude: number;
			description: string;
		}) => {
			const historyId = getHistoryId();
			const response = await authFetch(`${config.apiBaseUrl}/histories/${historyId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					car_id: data.carId,
					contact_id: data.contactId,
					activity_id: data.activityId,
					tracker_id: data.trackerId,
					started_at: data.startedAt,
					finished_at: data.finishedAt,
					finished_latitude: data.finishedLatitude,
					finished_longitude: data.finishedLongitude,
					description: data.description
				})
			});
			if (!response.ok) throw new Error("Failed to modify history");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/history"));
			await queryClient.invalidateQueries({ queryKey: ["histories"] });
		}
	}));
}

export function useDeleteHistoryMutation(getHistoryId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const historyId = getHistoryId();
			const response = await authFetch(`${config.apiBaseUrl}/histories/${historyId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete history");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/history"));
			await queryClient.invalidateQueries({ queryKey: ["histories"] });
		}
	}));
}

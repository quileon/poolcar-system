import type { GetTrackerResponse } from "$lib/bindings/GetTrackerResponse";
import type { TrackerWithDetails } from "$lib/bindings/TrackerWithDetails";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useTrackersQuery() {
	return createQuery<GetTrackerResponse>(() => ({
		queryKey: ["trackers"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/trackers`);
			if (!response.ok) throw new Error("Failed to fetch trackers");
			return response.json();
		}
	}));
}

export function useTrackerQuery(getTrackerId: () => number) {
	return createQuery<TrackerWithDetails>(() => ({
		queryKey: ["tracker", getTrackerId()],
		queryFn: async () => {
			const trackerId = getTrackerId();
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}`);
			if (!response.ok) throw new Error("Failed to fetch tracker");
			return response.json();
		}
	}));
}
export function useCreateTrackerMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/trackers`, {
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
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}`, {
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
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}`, {
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

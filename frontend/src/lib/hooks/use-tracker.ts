import type { GetTrackerResponse } from "$lib/bindings/GetTrackerResponse";
import type { TrackerDetails } from "$lib/bindings/TrackerDetails";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useTrackersQuery(getStatus: () => string | null) {
	return createQuery<GetTrackerResponse>(() => {
		const status = getStatus();
		const searchParams = new URLSearchParams();
		if (status) {
			searchParams.set("status", status);
		}

		return {
			queryKey: ["trackers", status],
			queryFn: async () => {
				const response = await authFetch(
					`${config.apiBaseUrl}/trackers?${searchParams.toString()}`
				);
				if (!response.ok) throw new Error("Failed to fetch trackers");
				return response.json();
			}
		};
	});
}

export function useTrackerQuery(getTrackerId: () => number) {
	const trackerId = getTrackerId();

	return createQuery<TrackerDetails>(() => ({
		queryKey: ["tracker", getTrackerId()],
		queryFn: async () => {
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
	const trackerId = getTrackerId();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
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
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
			await queryClient.invalidateQueries({ queryKey: ["tracker", trackerId] });
			await goto(resolve("/trackers"));
		}
	}));
}

export function useDeleteTrackerMutation(getTrackerId: () => number) {
	const queryClient = useQueryClient();
	const trackerId = getTrackerId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete tracker");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
			await queryClient.invalidateQueries({ queryKey: ["tracker", trackerId] });
			await goto(resolve("/trackers"));
		}
	}));
}

export function useRestoreTrackerMutation(getTrackerId: () => number) {
	const queryClient = useQueryClient();
	const trackerId = getTrackerId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/trackers/${trackerId}/restore`, {
				method: "PUT"
			});
			if (!response.ok) throw new Error("Failed to delete tracker");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
			await queryClient.invalidateQueries({ queryKey: ["tracker", trackerId] });
			await goto(resolve("/trackers"));
		}
	}));
}

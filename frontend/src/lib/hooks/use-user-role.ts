import type { UserRoleWithDetails } from "$lib/bindings/UserRoleWithDetails";
import type { GetUserRolesResponse } from "$lib/bindings/GetUserRolesResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useUserRolesQuery(getStatus: () => string | null) {
	return createQuery<GetUserRolesResponse[]>(() => {
		const status = getStatus();
		const searchParams = new URLSearchParams();
		if (status) {
			searchParams.set("status", status);
		}

		return {
			queryKey: ["user-roles", status],
			queryFn: async () => {
				const response = await authFetch(
					`${config.apiBaseUrl}/users/roles?${searchParams.toString()}`
				);
				if (!response.ok) throw new Error("Failed to fetch user roles");
				return response.json();
			}
		};
	});
}

export function useUserRoleQuery(getUserRoleId: () => number) {
	return createQuery<UserRoleWithDetails>(() => ({
		queryKey: ["user-role", getUserRoleId()],
		queryFn: async () => {
			const userRoleId = getUserRoleId();
			const response = await authFetch(`${config.apiBaseUrl}/users/roles/${userRoleId}`);
			if (!response.ok) throw new Error("Failed to fetch user role");
			return response.json();
		}
	}));
}

export function useCreateUserRoleMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/users/roles`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to create user role");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/user-roles"));
			await queryClient.invalidateQueries({ queryKey: ["user-roles"] });
		}
	}));
}

export function useEditUserRoleMutations(getUserRoleId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const userRoleId = getUserRoleId();
			const response = await authFetch(`${config.apiBaseUrl}/users/roles/${userRoleId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify user role");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/user-roles"));
			await queryClient.invalidateQueries({ queryKey: ["user-roles"] });
		}
	}));
}

export function useDeleteUserRoleMutation(getUserRoleId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const userRoleId = getUserRoleId();
			const response = await authFetch(`${config.apiBaseUrl}/users/roles/${userRoleId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete user role");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/user-roles"));
			await queryClient.invalidateQueries({ queryKey: ["users-roles"] });
		}
	}));
}

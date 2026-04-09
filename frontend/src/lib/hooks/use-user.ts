import type { GetUsersResponse } from "$lib/bindings/GetUsersResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import type { UserDetails } from "$lib/bindings/UserDetails";

export function useUsersQuery(getStatus: () => string | null) {
	return createQuery<GetUsersResponse>(() => {
		const status = getStatus();
		const searchParams = new URLSearchParams();
		if (status) {
			searchParams.set("status", status);
		}

		return {
			queryKey: ["users", status],
			queryFn: async () => {
				const response = await authFetch(`${config.apiBaseUrl}/users?${searchParams.toString()}`);
				if (!response.ok) throw new Error("Failed to fetch users");
				return response.json();
			}
		};
	});
}

export function useUserQuery(getUserId: () => number) {
	return createQuery<UserDetails>(() => ({
		queryKey: ["user", getUserId()],
		queryFn: async () => {
			const userId = getUserId();
			const response = await authFetch(`${config.apiBaseUrl}/users/${userId}`);
			if (!response.ok) throw new Error("Failed to fetch user");
			return response.json();
		}
	}));
}

export function useCreateUserMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			username: string;
			password: string | null;
			email: string;
			fullName: string;
			userRoleId: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/users`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					username: data.username,
					password: data.password,
					email: data.email,
					full_name: data.fullName,
					user_role_id: data.userRoleId
				})
			});
			if (!response.ok) throw new Error("Failed to create user");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/users"));
			await queryClient.invalidateQueries({ queryKey: ["users"] });
		}
	}));
}

export function useEditUserMutations(getUserId: () => number) {
	const queryClient = useQueryClient();
	const userId = getUserId();

	return createMutation(() => ({
		mutationFn: async (data: {
			username: string;
			password: string | null;
			email: string;
			fullName: string;
			userRoleId: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/users/${userId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					username: data.username,
					password: data.password,
					email: data.email,
					full_name: data.fullName,
					user_role_id: data.userRoleId
				})
			});
			if (!response.ok) throw new Error("Failed to modify user");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["users"] });
			await queryClient.invalidateQueries({ queryKey: ["user", userId] });
			await goto(resolve("/users"));
		}
	}));
}

export function useDeleteUserMutation(getUserId: () => number) {
	const queryClient = useQueryClient();
	const userId = getUserId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/users/${userId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete user");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["users"] });
			await queryClient.invalidateQueries({ queryKey: ["user", userId] });
			await goto(resolve("/users"));
		}
	}));
}

export function useRestoreUserMutation(getUserId: () => number) {
	const queryClient = useQueryClient();
	const userId = getUserId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/users/${userId}/restore`, {
				method: "PUT"
			});
			if (!response.ok) throw new Error("Failed to restore user");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["users"] });
			await queryClient.invalidateQueries({ queryKey: ["user", userId] });
			await goto(resolve("/users"));
		}
	}));
}

<script lang="ts">
	import { goto } from "$app/navigation";
	import { resolve } from "$app/paths";
	import { page } from "$app/state";
	import type { CarTypeWithCount } from "$lib/bindings/CarTypeWithCount";
	import { config } from "$lib/config";
	import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";

	const carTypeId = $derived(parseInt(page.params.id!, 10));
	const carTypeQuery = createQuery<CarTypeWithCount>(() => ({
		queryKey: ["car-types"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`);
			if (!response.ok) throw new Error("Failed to fetch car type");
			return response.json();
		}
	}));
	let carTypeName = $state("");
	$effect(() => {
		if (carTypeQuery.data) {
			carTypeName = carTypeQuery.data.name;
		}
	});
	const queryClient = useQueryClient();
	const editCarTypeMutation = createMutation(() => ({
		mutationFn: async (carTypeName: string) => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({ name: carTypeName })
			});
			if (!response.ok) {
				throw new Error("Failed to modify car type");
			}
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/car-types"));
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
		}
	}));
	const deleteCarTypeMutation = createMutation(() => ({
		mutationFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types/${carTypeId}`, {
				method: "DELETE"
			});
			if (!response.ok) {
				throw new Error("Failed to delete car type");
			}
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/car-types"));
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
		}
	}));

	function handleSubmit(event: Event) {
		event.preventDefault();
		editCarTypeMutation.mutate(carTypeName);
	}

	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${carTypeName}"?`)) {
			deleteCarTypeMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Modify Car Type</Field.Legend>
				<Field.Description>Car Type will be used to categorize cars.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Type Name</Field.Label>
						<Input
							id="name"
							bind:value={carTypeName}
							name="name"
							type="text"
							placeholder="Enter car type name"
							disabled={carTypeQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editCarTypeMutation.isPending ||
							carTypeQuery.isPending ||
							deleteCarTypeMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editCarTypeMutation.isPending ||
							carTypeQuery.isPending ||
							deleteCarTypeMutation.isPending}
						href="/car-types">Cancel</Button
					>
				</div>
				<Button
					type="button"
					disabled={editCarTypeMutation.isPending ||
						carTypeQuery.isPending ||
						deleteCarTypeMutation.isPending}
					variant="destructive"
					onclick={handleDelete}>Delete</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if carTypeQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{carTypeQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editCarTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editCarTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteCarTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteCarTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>

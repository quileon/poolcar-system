<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import {
		useCarTypeQuery,
		useDeleteCarTypeMutation,
		useEditCarTypeMutation
	} from "$lib/hooks/user-car-type";

	const carTypeId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const carTypeQuery = useCarTypeQuery(() => carTypeId);

	// Mutations
	const editCarTypeMutation = useEditCarTypeMutation(() => carTypeId);
	const deleteCarTypeMutation = useDeleteCarTypeMutation(() => carTypeId);

	// Form state
	let carTypeName = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (carTypeQuery.data) {
			carTypeName = carTypeQuery.data.name;
		}
	});

	function handleSubmit(event: Event) {
		event.preventDefault();
		editCarTypeMutation.mutate({ name: carTypeName });
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

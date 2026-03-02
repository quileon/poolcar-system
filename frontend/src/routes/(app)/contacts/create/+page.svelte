<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { useContactTypesQuery } from "$lib/hooks/use-contact-type";
	import { useCreateContactMutation } from "$lib/hooks/use-contact";
	import { resolve } from "$app/paths";

	let name = $state("");
	let latitude = $state("");
	let longitude = $state("");
	let contactTypeId = $state("");

	const contactTypesQuery = useContactTypesQuery();
	const createContactMutation = useCreateContactMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createContactMutation.mutate({
			name,
			latitude: Number.parseFloat(latitude),
			longitude: Number.parseFloat(longitude),
			contactTypeId: Number.parseInt(contactTypeId, 10)
		});
	}

	const contactTypeTrigger = $derived(
		contactTypesQuery.data?.contact_types.find(
			(contactType) => contactType.contact_type_id.toString() === contactTypeId
		)?.name ?? "Select Contact Type"
	);
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Contact</Field.Legend>
				<Field.Description>Contact is used for identifying car destination.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Contact Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							type="text"
							placeholder="Enter contact name"
							required
						/>
					</Field.Field>

					<div class="flex gap-4">
						<Field.Field>
							<Field.Label for="latitude">Latitude</Field.Label>
							<Input
								id="latitude"
								bind:value={latitude}
								type="text"
								placeholder="Enter contact latitude"
								required
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="longitude">Longitude</Field.Label>
							<Input
								id="longitude"
								bind:value={longitude}
								type="text"
								placeholder="Enter contact longitude"
								required
							/>
						</Field.Field>
					</div>

					<Field.Field>
						<Field.Label for="contact_type_id">Contact Type</Field.Label>
						<Select.Root type="single" bind:value={contactTypeId}>
							<Select.Trigger id="contact_type_id">{contactTypeTrigger}</Select.Trigger>
							<Select.Content>
								{#if contactTypesQuery.data?.contact_types}
									{#each contactTypesQuery.data.contact_types as contactType (contactType.contact_type_id)}
										<Select.Item value={contactType.contact_type_id.toString()}>
											{contactType.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description>Enter the type of contact.</Field.Description>
					</Field.Field>
				</Field.Group>
			</Field.Set>

			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createContactMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createContactMutation.isPending}
					href={resolve("/contacts")}
					>Cancel
				</Button>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if contactTypesQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Contact Type</Alert.Title>
				<Alert.Description>
					<p>{contactTypesQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if createContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Contact</Alert.Title>
				<Alert.Description>
					<p>{createContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>

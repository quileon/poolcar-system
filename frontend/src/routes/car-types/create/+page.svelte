<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
</script>

<div class="mx-auto w-full max-w-md">
	<form>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car Types</Field.Legend>
				<Field.Description>Car Type will be used to categorize cars.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Type Name</Field.Label>
						<Input id="name" name="name" type="text" placeholder="Enter car type name" required />
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit">Submit</Button>
				<Button variant="outline" type="button">Cancel</Button>
			</Field.Field>
		</Field.Group>
	</form>
</div>

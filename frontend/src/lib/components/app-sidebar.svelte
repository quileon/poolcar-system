<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar";
	import HouseIcon from "@lucide/svelte/icons/house";
	import CarIcon from "@lucide/svelte/icons/car";
	import CarFrontIcon from "@lucide/svelte/icons/car-front";
	import MapPinPenIcon from "@lucide/svelte/icons/map-pin-pen";
	import NotebookIcon from "@lucide/svelte/icons/notebook";
	import NotebookPenIcon from "@lucide/svelte/icons/notebook-pen";
	import BookOpenIcon from "@lucide/svelte/icons/book-open";
	import HistoryIcon from "@lucide/svelte/icons/history";
	import MapIcon from "@lucide/svelte/icons/map";
	import { resolve } from "$app/paths";
	import type { Component, ComponentProps } from "svelte";
	import type { Url } from "$lib/url";
	const sidebar_items: {
		title: string;
		url: Url;
		icon: Component;
	}[] = [
		{
			title: "Home",
			url: "/",
			icon: HouseIcon
		},
		{
			title: "Cars",
			url: "/cars",
			icon: CarIcon
		},
		{
			title: "Car Types",
			url: "/car-types",
			icon: CarFrontIcon
		},
		{
			title: "Trackers",
			url: "/trackers",
			icon: MapPinPenIcon
		},
		{
			title: "Contacts",
			url: "/contacts",
			icon: NotebookIcon
		},
		{
			title: "Contact Types",
			url: "/contact-types",
			icon: NotebookPenIcon
		},
		{
			title: "Activities",
			url: "/activity",
			icon: HistoryIcon
		},
		{
			title: "Activity Types",
			url: "/activity-types",
			icon: BookOpenIcon
		},
		{
			title: "Live Tracking",
			url: "/live",
			icon: MapIcon
		}
	];

	let { ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root collapsible="offcanvas" {...restProps}>
	<Sidebar.Header />
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Poolcar System</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each sidebar_items as sidebar_item (sidebar_item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={resolve(`${sidebar_item.url}`)} {...props}>
										<sidebar_item.icon />
										<span>{sidebar_item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer />
</Sidebar.Root>

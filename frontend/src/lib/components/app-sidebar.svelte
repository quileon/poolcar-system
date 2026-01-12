<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar";
	import HouseIcon from "@lucide/svelte/icons/house";
	import CarIcon from "@lucide/svelte/icons/car";
	import CarFrontIcon from "@lucide/svelte/icons/car-front";
	import MapPinPen from "@lucide/svelte/icons/map-pin-pen";
	import Notebook from "@lucide/svelte/icons/notebook";
	import NotebookPen from "@lucide/svelte/icons/notebook-pen";
	import BookOpen from "@lucide/svelte/icons/book-open";
	import History from "@lucide/svelte/icons/history";
	import Map from "@lucide/svelte/icons/map";
	import { resolve } from "$app/paths";
	import type { Component } from "svelte";
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
			icon: MapPinPen
		},
		{
			title: "Contacts",
			url: "/contacts",
			icon: Notebook
		},
		{
			title: "Contact Types",
			url: "/contact-types",
			icon: NotebookPen
		},
		{
			title: "Activity",
			url: "/activity",
			icon: BookOpen
		},
		{
			title: "History",
			url: "/history",
			icon: History
		},
		{
			title: "Live Tracking",
			url: "/live",
			icon: Map
		}
	];
</script>

<Sidebar.Root>
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

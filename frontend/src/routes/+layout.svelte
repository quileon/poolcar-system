<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import AppHeader from '$lib/components/app-header.svelte';
	import Separator from '$lib/components/ui/separator/separator.svelte';

	let sidebarOpen = $state(true);

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<Sidebar.Provider bind:open={sidebarOpen}>
	<AppSidebar />
	<Sidebar.Inset>
		<AppHeader {toggleSidebar} />
		<Separator />
		<main class="flex flex-1 flex-col gap-8 p-8">
			{@render children()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>

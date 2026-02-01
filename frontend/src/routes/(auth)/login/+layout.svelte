<script lang="ts">
	import "../../layout.css";
	import favicon from "$lib/assets/favicon.svg";
	import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";

	const queryClient = new QueryClient();

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<QueryClientProvider client={queryClient}>
	<main class="flex h-screen flex-row items-center justify-center p-8">
		{@render children()}
	</main>
</QueryClientProvider>

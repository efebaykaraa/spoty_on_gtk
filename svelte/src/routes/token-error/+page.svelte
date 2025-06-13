<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	
	let mounted = false;
	let errorMessage = '';
	
	onMount(() => {
		mounted = true;
		errorMessage = $page.url.searchParams.get('message') || 'Token exchange failed';
	});
</script>

<svelte:head>
	<title>Token Exchange Error - Spoty</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-red-500 to-red-600 flex items-center justify-center p-4">
	<div class="max-w-md w-full bg-white/10 backdrop-blur-lg rounded-2xl p-8 text-center text-white shadow-2xl border border-white/20" class:animate-fade-in={mounted}>
		<div class="text-6xl mb-6">🔒</div>
		<h1 class="text-3xl font-bold mb-4">Token Exchange Failed</h1>
		<div class="space-y-4">
			<div class="bg-white/20 rounded-lg p-4">
				<p class="font-semibold mb-2">🔑 Authentication Error</p>
				<p class="text-sm mb-2">Failed to exchange authorization code for access token.</p>
				<div class="bg-black/20 p-2 rounded text-xs font-mono text-left">
					{errorMessage}
				</div>
			</div>
			<div class="text-lg">
				<p>This could be due to:</p>
				<ul class="text-sm mt-2 text-left space-y-1 bg-white/10 p-3 rounded">
					<li>• Invalid client credentials</li>
					<li>• Expired authorization code</li>
					<li>• Network connectivity issues</li>
					<li>• Server configuration problems</li>
				</ul>
				<p class="font-bold mt-4">You can now close this window and return to the app.</p>
			</div>
		</div>
		<div class="flex gap-2 mt-6">
			<button 
				class="flex-1 bg-white/20 hover:bg-white/30 px-4 py-2 rounded-lg transition-colors"
				on:click={() => window.location.reload()}
			>
				Try Again
			</button>
			<button 
				class="flex-1 bg-white/20 hover:bg-white/30 px-4 py-2 rounded-lg transition-colors"
				on:click={() => window.close()}
			>
				Close Window
			</button>
		</div>
	</div>
</div>

<style>
	.animate-fade-in {
		animation: fadeIn 0.6s ease-out;
	}
	
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>

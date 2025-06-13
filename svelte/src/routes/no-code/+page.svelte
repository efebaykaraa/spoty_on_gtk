<script lang="ts">
	import { onMount } from 'svelte';
	
	let mounted = false;
	
	onMount(() => {
		mounted = true;
	});
</script>

<svelte:head>
	<title>Authorization Error - Spoty</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-red-500 to-red-600 flex items-center justify-center p-4">
	<div class="max-w-md w-full bg-white/10 backdrop-blur-lg rounded-2xl p-8 text-center text-white shadow-2xl border border-white/20" class:animate-fade-in={mounted}>
		<div class="text-6xl mb-6">❌</div>
		<h1 class="text-3xl font-bold mb-4">Authorization Failed</h1>
		<div class="space-y-4">
			<div class="bg-white/20 rounded-lg p-4">
				<p class="font-semibold mb-2">⚠️ Missing Authorization Code</p>
				<p class="text-sm">No authorization code was received from Spotify. This usually happens when:</p>
				<ul class="text-sm mt-2 text-left space-y-1">
					<li>• The authorization was cancelled</li>
					<li>• The redirect URL is incorrect</li>
					<li>• Network connectivity issues</li>
				</ul>
			</div>
			<div class="text-lg">
				<p>Please try the authorization process again.</p>
				<p class="font-bold mt-4">You can now close this window and return to the app.</p>
			</div>
		</div>
		<button 
			class="mt-6 bg-white/20 hover:bg-white/30 px-6 py-2 rounded-lg transition-colors"
			on:click={() => window.close()}
		>
			Close Window
		</button>
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

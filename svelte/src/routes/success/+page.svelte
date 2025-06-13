<script lang="ts">
	import { onMount } from 'svelte';
	
	let mounted = false;
	
	onMount(() => {
		mounted = true;
		// Auto-close window after 3 seconds
		setTimeout(() => {
			if (window.opener) {
				window.close();
			}
		}, 3000);
	});
</script>

<svelte:head>
	<title>Authorization Successful - Spoty</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-green-500 to-green-600 flex items-center justify-center p-4">
	<div class="max-w-md w-full bg-white/10 backdrop-blur-lg rounded-2xl p-8 text-center text-white shadow-2xl border border-white/20" class:animate-fade-in={mounted}>
		<div class="text-6xl mb-6 animate-bounce">🎵</div>
		<h1 class="text-3xl font-bold mb-4">Authorization Successful!</h1>
		<div class="space-y-4 text-lg">
			<p>You have successfully authorized the Spoty application to access your Spotify account.</p>
			<div class="bg-white/20 rounded-lg p-4">
				<p class="font-semibold">✅ Connected to Spotify</p>
				<p class="text-sm opacity-90 mt-2">This window will close automatically in a few seconds.</p>
			</div>
			<p class="font-bold">You can now close this window and return to the app.</p>
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

<script setup>
import Card from '../components/Card.vue';
</script>

<template>
	<div class="p-6">
		<div class="flex flex-col items-center mb-6">
			<h1 class="text-[#607B9B] text-3xl font-semibold mb-3 mt-5 text-center">CHOOSE A BOARD</h1>
		</div>

		<!-- Centered full-height wrapper -->
		<div class="min-h-[60vh] flex items-center justify-center">
			<Card class="w-[36rem] max-w-full">
				<div class="text-center">
					<h2 class="text-lg font-semibold text-[#324A5E] mb-2">KNOWN BOARDS</h2>
					<p class="text-sm text-[#64748B]">Select a known board:</p>
				</div>

				<!-- Known boards list: each RouterLink on its own row -->
				<div class="mt-4 flex flex-col items-stretch gap-2">
					<RouterLink to="/device-manager" class="block px-4 py-2 rounded bg-slate-50 text-[#324A5E] border border-slate-200">NucleoF4</RouterLink>
					<RouterLink to="/device-manager" class="block px-4 py-2 rounded bg-slate-50 text-[#324A5E] border border-slate-200">MicrobitV2</RouterLink>
					<RouterLink to="/device-manager" class="block px-4 py-2 rounded bg-slate-50 text-[#324A5E] border border-slate-200">Other boards</RouterLink>
				</div>

				<!-- Buttons on a single non-wrapping row; buttons share available space -->
				<div class="mt-4 flex flex-row flex-nowrap items-center justify-center gap-3 w-full">
					<router-link to="/customize-debug" class="flex-1 min-w-0">
						<button class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Customize Probe</button>
					</router-link>
					<router-link to="/" class="flex-1 min-w-0">
						<button class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Go Back</button>
					</router-link>
					<router-link to="/customize-serial" class="flex-1 min-w-0">
						<button class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Customize Serial</button>
					</router-link>
				</div>
			</Card>
		</div>
	</div>
</template>

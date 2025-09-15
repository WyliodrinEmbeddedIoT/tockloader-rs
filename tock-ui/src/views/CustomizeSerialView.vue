<script setup>
import { ref } from 'vue';
import Card from '../components/Card.vue';

const chip = ref('');
const core = ref('');

function onSubmit(e) {
	e.preventDefault();
	// For now just print values to console; later wire to store or API
	console.log('chip:', chip.value, 'core:', core.value);
}
</script>

<template>
	<div class="p-6">
		<!-- Centered full-height wrapper -->
		<div class="min-h-[60vh] flex items-center justify-center">
			<Card class="w-[40rem] max-w-full">
				<div class="text-center">
					<h1 class="text-[#607B9B] text-3xl font-semibold mb-3 mt-3 text-center">CUSTOMIZE SERIAL PORT SETTINGS</h1>
				</div>

                <p>For Port: /dev/ttyACM0</p>

				<!-- Simple form for chip and core -->
				<form @submit="onSubmit" class="mt-4 flex flex-col gap-3 text-left">
					<label>
						<span class="block">Baud Rate:</span>
						<input v-model="baud_rate" type="text" placeholder="e.g. 115200" class="mt-1 block w-full rounded border px-3 py-2" />
					</label>

					<label>
						<span class="block">Parity:</span>
						<input v-model="parity" type="text" placeholder="e.g. None" class="mt-1 block w-full rounded border px-3 py-2" />
					</label>

                    <label>
						<span class="block">Stop Bits:</span>
						<input v-model="stop_bits" type="text" placeholder="e.g. One" class="mt-1 block w-full rounded border px-3 py-2" />
					</label>

                    <label>
						<span class="block">Flow Control:</span>
						<input v-model="flow_control" type="text" placeholder="e.g. None" class="mt-1 block w-full rounded border px-3 py-2" />
					</label>

                    <label>
						<span class="block">Timeout(ms):</span>
						<input v-model="timeout" type="text" placeholder="e.g. 1000" class="mt-1 block w-full rounded border px-3 py-2" />
					</label>

                    <div class="mt-4 mb-3 flex flex-row flex-nowrap items-center justify-center gap-3 w-full">
					    <router-link to="/device-manager" class="flex-1 min-w-0">
						    <button type="submit" class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Connect</button>
					    </router-link>
					    <router-link to="/known-board" class="flex-1 min-w-0">
						    <button class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Save Settings</button>
					    </router-link>
					    <router-link to="/known-board" class="flex-1 min-w-0">
						    <button class="w-full bg-[#84A1C4] text-white px-3 py-2 rounded-full flex items-center justify-center">Cancel</button>
					    </router-link>
				    </div>
				</form>
				<!-- Buttons on a single non-wrapping row; buttons share available space -->
				
			</Card>
		</div>
	</div>
</template>

<script setup>
import Card from '../components/Card.vue';
</script>

<template>
	<div class="p-6">
		<div class="flex flex-col items-center mb-6">
			<h1 class="text-[#607B9B] text-3xl font-semibold mb-3 mt-5 text-center">CONNECTED DEVICES</h1>
			<button class="bg-[#607B9B] text-white px-3 py-2 rounded-full w-44 flex items-center justify-center">Refresh Devices</button>
		</div>

		<div class="flex gap-4 flex-wrap">
			<Card class="flex-1 min-w-[280px]">
                <h1 class="text-[#607B9B] text-2xl font-semibold mb-14 text-center">DEBUG PROBES</h1>
                <Card class="min-h-[230px] flex flex-col justify-between">
                    <div>
                        <p>Identifier: 12345</p>
                        <p>Vendor ID: 67890</p>
                        <p>Product ID: JTAG</p>
                        <p>Serial Number: 54321</p>
                    </div>

                    <div class="mt-4 self-end">
                        <router-link to="/known-board" class="inline-block">
                            <button class="bg-[#84A1C4] text-white px-3 py-2 rounded-full w-40 flex items-center justify-center">Connect</button>
                        </router-link>
                    </div>
                </Card>
			</Card>

			<Card class="flex-1 min-w-[280px]">
				<h1 class="text-[#607B9B] text-2xl font-semibold mb-3 text-center">SERIAL PORTS</h1>
                <div class="flex items-center justify-between mb-4">
                    <div></div>
                    <div class="inline-flex items-center gap-2 mt-1">
                        <span class="font-medium text-gray-700">Show/Hide</span>
                        <label class="relative inline-flex items-center cursor-pointer">
                            <input type="checkbox" class="sr-only peer" />
                            <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-checked:bg-[#607B9B] peer-focus:ring-2 peer-focus:ring-[#607B9B] peer-checked:after:translate-x-5 after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:border after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                        </label>
                    </div>
                </div>

                <Card>
                    <p>Port Name: 12345</p>
                    <p>USB VID: 67890</p>
                    <p>USB PID: 79865</p>
                    <p>Manufacturer: Example Manufacturer</p>
                    <p>Product Name: Example Product</p>
                    <p>Serial Number: ABCD1234</p>

                    <div class="flex items-center justify-between mt-4">
                        <router-link to="/known-board" class="inline-block">
                            <button class="bg-[#84A1C4] text-white px-3 py-2 rounded-full w-40 flex items-center justify-center">Connect</button>
                        </router-link>
                    </div>
                </Card>
                <Card>
                    <p>Port Name: 12345</p>
                    <p>USB VID: 67890</p>
                    <p>USB PID: 79865</p>
                    <p>Manufacturer: Example Manufacturer</p>
                    <p>Product Name: Example Product</p>
                    <p>Serial Number: ABCD1234</p>

                    <div class="flex items-center justify-between mt-4">
                        <router-link to="/known-board" class="inline-block">
                            <button class="bg-[#84A1C4] text-white px-3 py-2 rounded-full w-40 flex items-center justify-center">Connect</button>
                        </router-link>
                    </div>
                </Card> 
			</Card>
		</div>
	</div>
</template>

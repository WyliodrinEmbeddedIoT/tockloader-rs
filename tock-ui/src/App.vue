<!-- src/App.vue -->
<template>
  <div id="app" class="device-list-container">
    <h1 class="main-title">Connected Devices</h1>
    <button @click="getAllConnectedDevices" :disabled="loading" class="refresh-button">
      <span v-if="loading">Refreshing...</span>
      <span v-else>Refresh Devices</span>
    </button>

    <div v-if="loading" class="loading-message">Loading devices...</div>
    <div v-else-if="error" class="error-message">Error: {{ error }}</div>
    <div v-else class="device-sections">
      <section class="device-section">
        <h2 class="section-title">Debug Probes:</h2>
        <ul v-if="devices.debug_probes && devices.debug_probes.length > 0" class="device-list">
          <li v-for="probe in devices.debug_probes" :key="probe.identifier" class="device-item">
            <strong>Identifier:</strong> {{ probe.identifier }}<br>
            <strong>Vendor ID:</strong> {{ probe.vendor_id }}<br>
            <strong>Product ID:</strong> {{ probe.product_id }}<br>
            <strong>Serial Number:</strong> {{ probe.serial_number || 'N/A' }}
          </li>
        </ul>
        <p v-else class="no-devices-message">No debug probes found.</p>
      </section>

      <section class="device-section">
        <h2 class="section-title">Serial Ports:</h2>
        <!-- New toggle for /dev/ttySx ports -->
        <div class="toggle-all-tty-container">
          <label for="toggle-tty-ports" class="toggle-label">
            <input
              type="checkbox"
              id="toggle-tty-ports"
              v-model="hideTtySPorts"
              class="toggle-checkbox"
            />
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-text">{{ hideTtySPorts ? 'Hide /dev/ttySx Ports' : 'Show /dev/ttySx Ports' }}</span>
        </div>

        <ul v-if="devices.serial_ports && devices.serial_ports.length > 0" class="device-list">
          <li
            v-for="port in devices.serial_ports"
            :key="port.port_name"
            class="device-item"
            v-show="!(hideTtySPorts && port.port_name.startsWith('/dev/ttyS') && !isNaN(parseInt(port.port_name.substring(port.port_name.length - 1))))"
          >
            <strong>Port Name:</strong> {{ port.port_name }}<br>
            <strong>USB VID:</strong> {{ port.usb_vid || 'N/A' }}<br>
            <strong>USB PID:</strong> {{ port.usb_pid || 'N/A' }}<br>
            <strong>Manufacturer:</strong> {{ port.manufacturer || 'N/A' }}<br>
            <strong>Product:</strong> {{ port.product || 'N/A' }}<br>
            <strong>Serial Number:</strong> {{ port.serial_number || 'N/A' }}
          </li>
        </ul>
        <p v-else class="no-devices-message">No serial ports found.</p>
      </section>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Define TypeScript interfaces for the data received from Rust
interface DebugProbeSummary {
  identifier: string;
  vendor_id: number;
  product_id: number;
  serial_number?: string;
}

interface SerialPortSummary {
  port_name: string;
  usb_vid?: number;
  usb_pid?: number;
  manufacturer?: string;
  product?: string;
  serial_number?: string;
}

interface ConnectedDevices {
  debug_probes: DebugProbeSummary[];
  serial_ports: SerialPortSummary[];
}

export default defineComponent({
  name: 'App',
  setup() {
    // Reactive references for state management
    const devices = ref<ConnectedDevices>({ debug_probes: [], serial_ports: [] });
    const loading = ref<boolean>(false);
    const error = ref<string | null>(null);
    // New reactive state for hiding /dev/ttySx ports
    const hideTtySPorts = ref<boolean>(false); // Default to showing all ports

    // Function to call the Rust backend
    const getAllConnectedDevices = async () => {
      loading.value = true;
      error.value = null;
      try {
        const result = await invoke<ConnectedDevices>('list_all_devices');
        devices.value = result;
      } catch (err: any) {
        console.error('Error getting connected devices:', err);
        error.value = err.toString();
      } finally {
        loading.value = false;
      }
    };

    // Call the function when the component is mounted
    onMounted(() => {
      getAllConnectedDevices();
    });

    // Return reactive data and methods to the template
    return {
      devices,
      loading,
      error,
      hideTtySPorts, // Expose the new state
      getAllConnectedDevices,
    };
  },
});
</script>

<style>
body {
  font-family: 'Inter', sans-serif; 
  margin: 0;
  padding: 0;
  background-color: #f4f7f6;
  color: #333;
}

/* Container for the entire device list component */
#app.device-list-container {
  max-width: 1200px;
  margin: 40px auto;
  padding: 30px;
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.08);
  text-align: center;
}

.main-title {
  color: #2c3e50;
  font-size: 2.2em;
  margin-bottom: 25px;
  font-weight: 700;
}

.refresh-button {
  padding: 12px 25px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 1.1em;
  font-weight: 600;
  transition: background-color 0.3s ease, transform 0.2s ease;
  margin-bottom: 30px;
  box-shadow: 0 4px 10px rgba(0, 123, 255, 0.3);
}

.refresh-button:hover:not(:disabled) {
  background-color: #0056b3;
  transform: translateY(-2px);
}

.refresh-button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
  box-shadow: none;
}

.loading-message, .error-message {
  padding: 15px;
  border-radius: 8px;
  margin-top: 20px;
  font-size: 1.1em;
  font-weight: 500;
}

.loading-message {
  background-color: #e0f7fa;
  color: #00796b;
}

.error-message {
  background-color: #ffe0e0;
  color: #d32f2f;
}

.device-sections {
  display: flex;
  flex-wrap: wrap; /* Allows sections to wrap on smaller screens */
  gap: 25px;
  margin-top: 30px;
  justify-content: center;
}

.device-section {
  flex: 1; /* Allows sections to grow and shrink */
  min-width: 300px; /* Minimum width before wrapping */
  background-color: #fefefe;
  border: 1px solid #e0e0e0;
  border-radius: 10px;
  padding: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  text-align: left;
}

.section-title {
  color: #4a69bd;
  font-size: 1.6em;
  margin-bottom: 15px;
  border-bottom: 2px solid #4a69bd;
  padding-bottom: 10px;
  font-weight: 600;
}

.device-list {
  list-style-type: none;
  padding: 0;
  margin: 0;
}

.device-item {
  background-color: #f0f4f7;
  margin-bottom: 10px;
  padding: 15px;
  border-radius: 8px;
  border: 1px solid #e8e8e8;
  line-height: 1.6;
  font-size: 0.95em;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  position: relative; /* For positioning the toggle */
}

.device-item:hover {
  transform: translateY(-3px);
  box-shadow: 0 6px 15px rgba(0, 0, 0, 0.1);
}

.device-item strong {
  color: #34495e;
}

.no-devices-message {
  color: #777;
  font-style: italic;
  padding: 10px;
}

.toggle-container {
  display: flex;
  align-items: center;
  margin-top: 10px;
  gap: 10px;
  justify-content: flex-end; 
}

.toggle-all-tty-container { 
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 15px;
  justify-content: flex-start;
  padding-left: 5px;
}


.toggle-label {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 24px;
}

.toggle-checkbox {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
  border-radius: 24px; 
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

.toggle-checkbox:checked + .toggle-slider {
  background-color: #4CAF50; 
}

.toggle-checkbox:focus + .toggle-slider {
  box-shadow: 0 0 1px #4CAF50;
}

.toggle-checkbox:checked + .toggle-slider:before {
  transform: translateX(16px);
}

.toggle-text {
  font-size: 0.9em;
  color: #555;
  font-weight: 500;
}


@media (max-width: 768px) {
  #app.device-list-container {
    margin: 20px;
    padding: 20px;
  }
  .device-sections {
    flex-direction: column; /* Stack sections vertically on small screens */
    gap: 20px;
  }
  .device-section {
    min-width: unset; /* Remove min-width to allow full flexibility */
    width: 100%;
  }
  .main-title {
    font-size: 1.8em;
  }
  .section-title {
    font-size: 1.4em;
  }
  .refresh-button {
    font-size: 1em;
    padding: 10px 20px;
  }
}
</style>
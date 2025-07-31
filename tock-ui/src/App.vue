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

            <div v-if="probe.customChip || probe.customCore !== undefined" class="custom-settings-display">
              <strong>Custom Chip:</strong> {{ probe.customChip || 'N/A' }}<br>
              <strong>Custom Core:</strong> {{ probe.customCore !== undefined ? probe.customCore : 'N/A' }}
            </div>

            <div class="customize-button-wrapper">
              <button @click="openCustomizationModal(probe)" class="customize-button">
                Customize Settings
              </button>
            </div>
          </li>
        </ul>
        <p v-else class="no-devices-message">No debug probes found.</p>
      </section>

      <section class="device-section">
        <h2 class="section-title">Serial Ports:</h2>
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

            <div v-if="port.customSettings" class="custom-settings-display">
              <strong>Custom Baud Rate:</strong> {{ port.customSettings.baudRate || 'N/A' }}<br>
              <strong>Custom Parity:</strong> {{ port.customSettings.parity || 'N/A' }}<br>
            
            </div>
            <div class="customize-button-wrapper">
              <button @click="openSerialCustomizationModal(port)" class="customize-button">
                Customize Settings
              </button>
            </div>
          </li>
        </ul>
        <p v-else class="no-devices-message">No serial ports found.</p>
      </section>
    </div>
    <div v-if="showCustomizationModal" class="modal-overlay" @click.self="closeCustomizationModal">
      <div class="modal-content">
        <h3 class="modal-title">Customize Probe Settings</h3>
        <p class="modal-description">For Probe: <strong>{{ selectedProbe?.identifier }}</strong></p>

        <div class="form-group">
          <label for="custom-chip">Chip:</label>
          <input type="text" id="custom-chip" v-model="tempCustomChip" class="form-input" />
        </div>

        <div class="form-group">
          <label for="custom-core">Core:</label>
          <input type="number" id="custom-core" v-model.number="tempCustomCore" class="form-input" />
        </div>

        <div v-if="connectionStatus" :class="['connection-status', connectionStatus.type]">
          {{ connectionStatus.message }}
          <div v-if="connectionStatus.type === 'success' && connectionStatus.data">
            <h4>Connected to Probe</h4>
            <ul>
              <li v-for="(region, index) in connectionStatus.data.memory_map_summary" :key="index">
                {{ region }}
              </li>
            </ul>
          </div>
        </div>

        <div class="modal-actions">
          <button @click="connectToProbe" class="connect-button" :disabled="connecting">
            <span v-if="connecting">Connecting...</span>
            <span v-else>Connect</span>
          </button>
          <button @click="saveCustomSettings" class="save-button">Save Settings</button>
          <button @click="closeCustomizationModal" class="cancel-button">Cancel</button>
        </div>
      </div>
    </div>

    <div v-if="showSerialCustomizationModal" class="modal-overlay" @click.self="closeSerialCustomizationModal">
      <div class="modal-content">
        <h3 class="modal-title">Customize Serial Port Settings</h3>
        <p class="modal-description">For Port: <strong>{{ selectedSerialPortForConfig?.port_name }}</strong></p>

        <div class="form-group">
          <label for="serial-baud-rate">Baud Rate:</label>
          <input type="number" id="serial-baud-rate" v-model.number="tempBaudRate" class="form-input" />
        </div>

        <div class="form-group">
          <label for="serial-parity">Parity:</label>
          <select id="serial-parity" v-model="tempParity" class="form-input">
            <option value="None">None</option>
            <option value="Odd">Odd</option>
            <option value="Even">Even</option>
          </select>
        </div>

        <div class="form-group">
          <label for="serial-stop-bits">Stop Bits:</label>
          <select id="serial-stop-bits" v-model="tempStopBits" class="form-input">
            <option value="One">One</option>
            <option value="Two">Two</option>
          </select>
        </div>

        <div class="form-group">
          <label for="serial-flow-control">Flow Control:</label>
          <select id="serial-flow-control" v-model="tempFlowControl" class="form-input">
            <option value="None">None</option>
            <option value="Software">Software</option>
            <option value="Hardware">Hardware</option>
          </select>
        </div>

        <div class="form-group">
          <label for="serial-timeout-ms">Timeout (ms):</label>
          <input type="number" id="serial-timeout-ms" v-model.number="tempTimeoutMs" class="form-input" />
        </div>

        <div class="form-group flex items-center">
          <input type="checkbox" id="serial-rts" v-model="tempRequestToSend" class="mr-2" />
          <label for="serial-rts">Request To Send (RTS)</label>
        </div>

        <div class="form-group flex items-center">
          <input type="checkbox" id="serial-dtr" v-model="tempDataTerminalReady" class="mr-2" />
          <label for="serial-dtr">Data Terminal Ready (DTR)</label>
        </div>

        <div v-if="serialConnectionStatus" :class="['connection-status', serialConnectionStatus.type]">
          {{ serialConnectionStatus.message }}
        </div>

        <div class="modal-actions">
          <button @click="connectToSerialWithCustomSettings" class="connect-button" :disabled="connectingSerial">
            <span v-if="connectingSerial">Connecting...</span>
            <span v-else>Connect</span>
          </button>
          <button @click="saveSerialCustomSettings" class="save-button">Save Settings</button>
          <button @click="closeSerialCustomizationModal" class="cancel-button">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
interface SerialCustomSettings {
  baudRate: number;
  parity: string;
  stopBits: string;
  flowControl: string;
  timeoutMs: number;
  requestToSend: boolean;
  dataTerminalReady: boolean;
}

interface DebugProbeSummary {
  identifier: string;
  vendor_id: number;
  product_id: number;
  serial_number?: string;
  customChip?: string;
  customCore?: number;
}

interface SerialPortSummary {
  port_name: string;
  usb_vid?: number;
  usb_pid?: number;
  manufacturer?: string;
  product?: string;
  serial_number?: string;
  customSettings?: SerialCustomSettings; 
}

interface ConnectedDevices {
  debug_probes: DebugProbeSummary[];
  serial_ports: SerialPortSummary[];
}

interface ConnectedProbeInfo {
  message: string;
  target_name: string;
  target_architecture: string;
  memory_map_summary: string[];
}

export default defineComponent({
  name: 'App',
  setup() {
    const devices = ref<ConnectedDevices>({ debug_probes: [], serial_ports: [] });
    const loading = ref<boolean>(false);
    const error = ref<string | null>(null);
    const hideTtySPorts = ref<boolean>(false);

    const showCustomizationModal = ref<boolean>(false);
    const selectedProbe = ref<DebugProbeSummary | null>(null);
    const tempCustomChip = ref<string>('');
    const tempCustomCore = ref<number | undefined>(undefined);
    const connecting = ref<boolean>(false);
    const connectionStatus = ref<{ message: string; type: 'success' | 'error'; data?: ConnectedProbeInfo } | null>(null);

    const showSerialCustomizationModal = ref<boolean>(false);
    const selectedSerialPortForConfig = ref<SerialPortSummary | null>(null);
    const tempBaudRate = ref<number>(115200);
    const tempParity = ref<string>('None');
    const tempStopBits = ref<string>('One');
    const tempFlowControl = ref<string>('None');
    const tempTimeoutMs = ref<number>(500);
    const tempRequestToSend = ref<boolean>(false);
    const tempDataTerminalReady = ref<boolean>(false);
    const connectingSerial = ref<boolean>(false);
    const serialConnectionStatus = ref<{ message: string; type: 'success' | 'error' } | null>(null);
   

    const getAllConnectedDevices = async () => {
      loading.value = true;
      error.value = null;
      try {
        const result = await invoke<ConnectedDevices>('list_all_devices');
        const updatedProbes = result.debug_probes.map((probe: DebugProbeSummary) => {
          const existingProbe = devices.value.debug_probes.find(p => p.identifier === probe.identifier);
          return {
            ...probe,
            customChip: existingProbe?.customChip,
            customCore: existingProbe?.customCore,
          };
        });

        const updatedSerialPorts = result.serial_ports.map((port: SerialPortSummary) => {
            const existingPort = devices.value.serial_ports.find(p => p.port_name === port.port_name);
            return {
                ...port,
                customSettings: existingPort?.customSettings || { 
                    baudRate: 115200,
                    parity: 'None',
                    stopBits: 'One',
                    flowControl: 'None',
                    timeoutMs: 500,
                    requestToSend: false,
                    dataTerminalReady: false,
                }
            };
        });

        devices.value = { ...result, debug_probes: updatedProbes, serial_ports: updatedSerialPorts }; 
      } catch (err: any) {
        console.error('Error getting connected devices:', err);
        error.value = err.toString();
      } finally {
        loading.value = false;
      }
    };

    const openCustomizationModal = (probe: DebugProbeSummary) => {
      selectedProbe.value = probe;
      tempCustomChip.value = probe.customChip || '';
      tempCustomCore.value = probe.customCore;
      connectionStatus.value = null;
      showCustomizationModal.value = true;
    };

    const closeCustomizationModal = () => {
      showCustomizationModal.value = false;
      selectedProbe.value = null;
      tempCustomChip.value = '';
      tempCustomCore.value = undefined;
      connectionStatus.value = null;
    };

    const saveCustomSettings = () => {
      if (selectedProbe.value) {
        const index = devices.value.debug_probes.findIndex(p => p.identifier === selectedProbe.value?.identifier);
        if (index !== -1) {
          devices.value.debug_probes[index].customChip = tempCustomChip.value;
          devices.value.debug_probes[index].customCore = tempCustomCore.value !== null && tempCustomCore.value !== undefined && !isNaN(tempCustomCore.value)
            ? tempCustomCore.value
            : undefined;
        }
      }
      connectionStatus.value = { message: 'Settings saved locally!', type: 'success' };
    };

    const connectToProbe = async () => {
      if (!selectedProbe.value || !tempCustomChip.value || tempCustomCore.value === undefined || isNaN(tempCustomCore.value)) {
        connectionStatus.value = { message: 'Please provide a valid Chip and Core.', type: 'error' };
        return;
      }

      connecting.value = true;
      connectionStatus.value = null;
      try {
        const result = await invoke<ConnectedProbeInfo>('connect_to_probe', {
          probeIdentifier: selectedProbe.value.identifier,
          chip: tempCustomChip.value,
          core: tempCustomCore.value,
        });
        connectionStatus.value = { message: result.message, type: 'success', data: result };
      } catch (err: any) {
        console.error('Connection error:', err);
        connectionStatus.value = { message: err.toString(), type: 'error' };
      } finally {
        connecting.value = false;
      }
    };


    const openSerialCustomizationModal = (port: SerialPortSummary) => {
      selectedSerialPortForConfig.value = port;
      const currentSettings = port.customSettings || {
        baudRate: 115200,
        parity: 'None',
        stopBits: 'One',
        flowControl: 'None',
        timeoutMs: 500,
        requestToSend: false,
        dataTerminalReady: false,
      };
      tempBaudRate.value = currentSettings.baudRate;
      tempParity.value = currentSettings.parity;
      tempStopBits.value = currentSettings.stopBits;
      tempFlowControl.value = currentSettings.flowControl;
      tempTimeoutMs.value = currentSettings.timeoutMs;
      tempRequestToSend.value = currentSettings.requestToSend;
      tempDataTerminalReady.value = currentSettings.dataTerminalReady;

      serialConnectionStatus.value = null; 
      showSerialCustomizationModal.value = true;
    };

    const closeSerialCustomizationModal = () => {
      showSerialCustomizationModal.value = false;
      selectedSerialPortForConfig.value = null;
      serialConnectionStatus.value = null;
    };

    const saveSerialCustomSettings = () => {
      if (selectedSerialPortForConfig.value) {
        const index = devices.value.serial_ports.findIndex(p => p.port_name === selectedSerialPortForConfig.value?.port_name);
        if (index !== -1) {
          devices.value.serial_ports[index].customSettings = {
            baudRate: tempBaudRate.value,
            parity: tempParity.value,
            stopBits: tempStopBits.value,
            flowControl: tempFlowControl.value,
            timeoutMs: tempTimeoutMs.value,
            requestToSend: tempRequestToSend.value,
            dataTerminalReady: tempDataTerminalReady.value,
          };
        }
      }
      serialConnectionStatus.value = { message: 'Settings saved locally!', type: 'success' };
    };

    const connectToSerialWithCustomSettings = async () => {
      if (!selectedSerialPortForConfig.value) {
        serialConnectionStatus.value = { message: 'No serial port selected.', type: 'error' };
        return;
      }

      connectingSerial.value = true;
      serialConnectionStatus.value = null;
      try {
        const result = await invoke<string>('connect_to_serial', {
          portName: selectedSerialPortForConfig.value.port_name,
          baudRate: tempBaudRate.value,
          parityStr: tempParity.value,
          stopBitsStr: tempStopBits.value,
          flowControlStr: tempFlowControl.value,
          timeoutMs: tempTimeoutMs.value,
          requestToSend: tempRequestToSend.value,
          dataTerminalReady: tempDataTerminalReady.value,
        });
        serialConnectionStatus.value = { message: result, type: 'success' };
      } catch (err: any) {
        console.error('Serial connection error:', err);
        serialConnectionStatus.value = { message: err.toString(), type: 'error' };
      } finally {
        connectingSerial.value = false;
      }
    };

    onMounted(() => {
      getAllConnectedDevices();
    });

    return {
      devices,
      loading,
      error,
      hideTtySPorts,
      showCustomizationModal,
      selectedProbe,
      tempCustomChip,
      tempCustomCore,
      connecting,
      connectionStatus,
      getAllConnectedDevices,
      openCustomizationModal,
      closeCustomizationModal,
      saveCustomSettings,
      connectToProbe,
      showSerialCustomizationModal,
      selectedSerialPortForConfig,
      tempBaudRate,
      tempParity,
      tempStopBits,
      tempFlowControl,
      tempTimeoutMs,
      tempRequestToSend,
      tempDataTerminalReady,
      connectingSerial,
      serialConnectionStatus,
      openSerialCustomizationModal,
      closeSerialCustomizationModal,
      saveSerialCustomSettings,
      connectToSerialWithCustomSettings
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
  flex-wrap: wrap; 
  gap: 25px;
  margin-top: 30px;
  justify-content: center;
}

.device-section {
  flex: 1; 
  min-width: 300px; 
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
  position: relative; 
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

.custom-settings-display {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed #ddd;
  font-size: 0.85em;
  color: #666;
  text-align: left; 
}

.customize-button-wrapper {
  text-align: center; 
  margin-top: 15px; 
}

.customize-button {
  padding: 10px 20px; 
  background-color: #28a745;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  font-size: 1em; 
  transition: background-color 0.3s ease;
  display: inline-block; 
}

.customize-button:hover {
  background-color: #218838;
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

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: #ffffff;
  padding: 30px;
  border-radius: 10px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  width: 90%;
  max-width: 450px;
  text-align: left;
  animation: fadeIn 0.3s ease-out;
}

.modal-title {
  color: #2c3e50;
  font-size: 1.8em;
  margin-bottom: 15px;
  border-bottom: 1px solid #eee;
  padding-bottom: 10px;
}

.modal-description {
  color: #666;
  margin-bottom: 20px;
  font-size: 0.95em;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #34495e;
}

.form-input {
  width: calc(100% - 20px);
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 1em;
  box-sizing: border-box; 
}

.modal-actions {
  margin-top: 25px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.save-button, .cancel-button, .connect-button {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1em;
  transition: background-color 0.3s ease;
}

.save-button {
  background-color: #007bff;
  color: white;
}

.save-button:hover {
  background-color: #0056b3;
}

.cancel-button {
  background-color: #6c757d;
  color: white;
}

.cancel-button:hover {
  background-color: #5a6268;
}

.connect-button {
  background-color: #4a69bd;
  color: white;
}

.connect-button:hover:not(:disabled) {
  background-color: #3a5191;
}

.connect-button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.connection-status {
  margin-top: 15px;
  padding: 10px;
  border-radius: 6px;
  font-size: 0.9em;
  font-weight: 500;
  text-align: center;
}

.connection-status.success {
  background-color: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.connection-status.error {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}


@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-20px); }
  to { opacity: 1; transform: translateY(0); }
}

@media (max-width: 768px) {
  #app.device-list-container {
    margin: 20px;
    padding: 20px;
  }
  .device-sections {
    flex-direction: column; 
    gap: 20px;
  }
  .device-section {
    min-width: unset;
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
  .modal-content {
    padding: 20px;
  }
}
</style>

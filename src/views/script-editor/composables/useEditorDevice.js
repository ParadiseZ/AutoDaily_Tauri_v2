
import { ref } from 'vue';

export function useEditorDevice({ getAllDevices, getFromStore, setToStore, deviceKey }) {
    const devices = ref([]);
    const currentDevice = ref(null);

    const loadDevices = async () => {
        try {
            devices.value = await getAllDevices();
            // Load saved device from store
            const savedDeviceId = await getFromStore(deviceKey);
            if (savedDeviceId && devices.value.some(d => d.id === savedDeviceId)) {
                currentDevice.value = savedDeviceId;
            }
        } catch (error) {
            console.error('Failed to load devices in editor:', error);
        }
    };

    const selectDevice = async (deviceId) => {
        currentDevice.value = deviceId;
        await setToStore(deviceKey, deviceId);
    };

    return {
        devices,
        currentDevice,
        loadDevices,
        selectDevice
    };
}
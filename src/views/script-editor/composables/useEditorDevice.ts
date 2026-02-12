import { ref } from 'vue';
import type { DeviceTable } from '../../../types/bindings';

interface EditorDeviceOptions {
    getAllDevices: () => Promise<DeviceTable[]>;
    getFromStore: (key: string) => Promise<string | null>;
    setToStore: (key: string, value: string) => Promise<void>;
    deviceKey: string;
}

export function useEditorDevice(options: EditorDeviceOptions) {
    const { getAllDevices, getFromStore, setToStore, deviceKey } = options;
    const devices = ref<DeviceTable[]>([]);
    const currentDevice = ref<string | null>(null);

    const loadDevices = async () => {
        try {
            devices.value = await getAllDevices();
            const savedDeviceId = await getFromStore(deviceKey);
            if (savedDeviceId && devices.value.some(d => d.id === savedDeviceId)) {
                currentDevice.value = savedDeviceId;
            }
        } catch (error) {
            console.error('Failed to load devices in editor:', error);
        }
    };

    const selectDevice = async (deviceId: string) => {
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

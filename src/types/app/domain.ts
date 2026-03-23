import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { ScriptInfo } from '@/types/bindings/ScriptInfo';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type { JsonValue as StoreJsonValue } from '@/types/bindings/serde_json/JsonValue';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

export type JsonValue = StoreJsonValue;

export type AppTheme = 'light' | 'dark' | 'system';
export type DefaultRoute = '/' | '/tasks' | '/devices' | '/scripts' | '/market' | '/logs' | '/settings' | '/about';
export type StartMode = 'normal' | 'minimized' | 'tray';
export type IdleAction = 'none' | 'shutdown' | 'sleep' | 'hibernate';
export type DeviceStatusKind = 'idle' | 'running' | 'paused' | 'stopped' | 'error' | 'unknown';

export interface ShortCutConfig {
    toggleWindow: string;
    toggleAllScripts: string;
    capture: string;
}

export interface SystemPreferences {
    appTheme: AppTheme;
    defaultRoute: DefaultRoute;
    startMode: StartMode;
    closeExit: boolean;
    alwaysOnTop: boolean;
    autoStart: boolean;
    idleAction: IdleAction;
    maxIdleRetryNum: number;
    adbPath: string;
    adbServerHost: string;
    adbServerPort: number;
    shortcut: ShortCutConfig;
}

export interface SystemConfigPayload {
    startMode: StartMode;
    closeExit: boolean;
    alwaysOnTop: boolean;
    idleAction: IdleAction;
    maxIdleRetryNum: number;
    autoStart: boolean;
    shortcut: ShortCutConfig;
}

export interface LogConfig {
    logLevel: LogLevel;
    logDir: string;
    retentionDays: number;
}

export interface UserProfile {
    id: string;
    username: string;
    email: string;
    isDeveloper: boolean;
    lastScriptUploadTime: string;
    lastUsernameChangeTime: string;
    sponsorUntil?: string | null;
}

type RawScriptInfo = ScriptInfo & {
    scriptType?: ScriptType;
    scriptTyCpe?: ScriptType;
};

export interface ScriptInfoRecord extends Omit<RawScriptInfo, 'scriptTyCpe'> {
    scriptType: ScriptType;
}

export interface ScriptTableRecord {
    id: string;
    data: ScriptInfoRecord;
}

export interface AssignmentRecord extends Omit<DeviceScriptAssignment, 'accountData'> {
    accountData: JsonValue;
}

export interface DeviceRuntimeStatus {
    rawStatus: string;
    kind: DeviceStatusKind;
    currentScript?: string | null;
    message?: string | null;
}

export interface DeviceStatusEvent {
    deviceId: string;
    status: string;
    currentScript?: string | null;
    message?: string | null;
}

export interface DeviceLogEntry {
    deviceId: string;
    level: LogLevel | 'Trace';
    message: string;
    time: string;
}

export interface ScriptSearchInput {
    page: number;
    size: number;
    keyword?: string;
    author?: string;
    runtimeType?: string;
}

export interface MarketScriptRecord {
    id: string;
    name: string | null;
    description: string | null;
    userId: string | null;
    userName: string | null;
    runtimeType: string | null;
    sponsorshipQr: string | null;
    sponsorshipUrl: string | null;
    contactInfo: string | null;
    imgDetModel: string | null;
    txtDetModel: string | null;
    txtRecModel: string | null;
    pkgName: string | null;
    createTime: string | null;
    updateTime: string | null;
    verName: string | null;
    verNum: number | null;
    latestVer: number | null;
    downloadCount: number | null;
    scriptType: string | null;
    isValid: boolean | null;
    allowClone: boolean | null;
    cloudId: string | null;
}

export interface MarketPage<T> {
    records: T[];
    total: number;
    size: number;
    current: number;
}

export interface UpdatePlatformInfo {
    signature: string;
    url: string;
}

export interface UpdateInfo {
    version: string;
    notes: string;
    pubDate: string;
    platforms: Record<string, UpdatePlatformInfo>;
}

export interface DeviceFormState {
    id: string | null;
    deviceName: string;
    exePath: string;
    exeArgs: string;
    cores: number[];
    logLevel: LogLevel;
    logToFile: boolean;
    capMethodType: 'window' | 'adb';
    capMethodValue: string;
    connectMethod: 'directTcp' | 'directUsb' | 'serverConnectByIp' | 'serverConnectByName';
    connectAddress: string;
    connectDeviceName: string;
    enable: boolean;
    autoStart: boolean;
}

export interface DeviceSummary {
    total: number;
    enabled: number;
    online: number;
    running: number;
}

export interface DeviceOption {
    label: string;
    value: string;
}

export type DeviceRecord = DeviceTable;

export const DEFAULT_SHORTCUTS: ShortCutConfig = {
    toggleWindow: 'CommandOrControl+H',
    toggleAllScripts: 'Alt+R',
    capture: 'Alt+A',
};

export const DEFAULT_SYSTEM_PREFERENCES: SystemPreferences = {
    appTheme: 'system',
    defaultRoute: '/tasks',
    startMode: 'normal',
    closeExit: true,
    alwaysOnTop: false,
    autoStart: false,
    idleAction: 'none',
    maxIdleRetryNum: 3,
    adbPath: '',
    adbServerHost: '127.0.0.1',
    adbServerPort: 5037,
    shortcut: DEFAULT_SHORTCUTS,
};

export const DEFAULT_LOG_CONFIG: LogConfig = {
    logLevel: 'Info',
    logDir: 'logs',
    retentionDays: 7,
};

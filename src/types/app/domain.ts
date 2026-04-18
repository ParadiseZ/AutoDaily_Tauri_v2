import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { DevicePlatform } from '@/types/bindings/DevicePlatform';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { RuntimeType } from '@/types/bindings/RuntimeType';
import type { ScriptVariableCatalog } from '@/types/bindings/ScriptVariableCatalog';
import type { ScriptPlatform } from '@/types/bindings/ScriptPlatform';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type { TimeoutAction } from '@/types/bindings/TimeoutAction';
import type { TimeoutNotifyChannel } from '@/types/bindings/TimeoutNotifyChannel';
import type { JsonValue as StoreJsonValue } from '@/types/bindings/serde_json/JsonValue';

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
    ocrTextCacheEnabled: boolean;
    ocrTextCacheDir: string;
    visionSignatureGridSize: number;
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

export type EmailProviderPreset = 'custom' | '163' | 'qq' | 'gmail' | 'outlook';
export type EmailSecurity = 'tlsWrapper' | 'startTls' | 'none';

export interface EmailConfig {
    desktopNotice: boolean;
    emailNotification: boolean;
    provider: EmailProviderPreset;
    smtpServer: string;
    smtpPort: number;
    security: EmailSecurity;
    username: string;
    password: string;
    senderName: string;
    senderEmail: string;
    recipient: string;
    timeoutSeconds: number;
}

export interface VisionTextCacheConfig {
    enabled: boolean;
    dir: string;
    signatureGridSize: number;
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

export interface AuthSession {
    accessToken: string;
    refreshToken: string;
    username: string;
    message?: string | null;
}

export interface ScriptAuthorSeed {
    userId?: string | null;
    userName?: string | null;
}

export interface ScriptInfoRecord {
    name: string;
    description: string | null;
    userId: string;
    userName: string | null;
    runtimeType: RuntimeType;
    platform: ScriptPlatform;
    sponsorshipQr: string | null;
    sponsorshipUrl: string | null;
    contactInfo: string | null;
    imgDetModel: DetectorType | null;
    txtDetModel: DetectorType | null;
    txtRecModel: RecognizerType | null;
    pkgName: string | null;
    activityName: string | null;
    createTime: string | null;
    updateTime: string | null;
    verName: string;
    scriptType: ScriptType;
    verNum: number;
    latestVer: number;
    downloadCount: number;
    isValid: boolean;
    allowClone: boolean;
    variableCatalog: ScriptVariableCatalog;
    cloudId: string | null;
    runtimeSettings: {
        recoveryTaskId: string | null;
    };
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
    sessionId?: string | null;
    status: string;
    currentScript?: string | null;
    message?: string | null;
}

export type RunTarget =
    | { type: 'deviceQueue' }
    | { type: 'fullScript'; scriptId: string }
    | { type: 'task'; scriptId: string; taskId: string }
    | { type: 'policyGroup'; scriptId: string; policyGroupId: string }
    | { type: 'policySet'; scriptId: string; policySetId: string }
    | { type: 'policy'; scriptId: string; policyId: string };

export interface RuntimeLifecycleEvent {
    deviceId: string;
    sessionId?: string | null;
    status: string;
    currentScript?: string | null;
    message?: string | null;
    at?: string | null;
}

export interface RuntimeProgressEvent {
    deviceId: string;
    sessionId?: string | null;
    assignmentId?: string | null;
    scriptId?: string | null;
    taskId?: string | null;
    stepId?: string | null;
    phase: string;
    message?: string | null;
    at: string;
}

export interface RuntimeScheduleEvent {
    deviceId: string;
    sessionId?: string | null;
    executionId?: string | null;
    assignmentId?: string | null;
    scriptId?: string | null;
    taskId?: string | null;
    stepId?: string | null;
    status: string;
    message?: string | null;
    at: string;
}

export interface RuntimeTimeoutEvent {
    deviceId: string;
    sessionId?: string | null;
    assignmentId?: string | null;
    scriptId?: string | null;
    taskId?: string | null;
    stepId?: string | null;
    message: string;
    at: string;
}

export interface ScriptTimeTemplateValuesDto {
    id: string;
    deviceId?: string | null;
    scriptId: string;
    timeTemplateId: string;
    accountId?: string | null;
    valuesJson: JsonValue;
    createdAt: string;
    updatedAt: string;
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
    activityName: string | null;
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
    platform: DevicePlatform;
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
    actionWaitMs: number;
    progressTimeoutEnabled: boolean;
    progressTimeoutMs: number;
    timeoutAction: TimeoutAction;
    timeoutNotifyChannels: TimeoutNotifyChannel[];
}

export interface DeviceSummary {
    total: number;
    enabled: number;
    online: number;
    running: number;
}

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
    ocrTextCacheEnabled: false,
    ocrTextCacheDir: '',
    visionSignatureGridSize: 8,
    shortcut: DEFAULT_SHORTCUTS,
};

export const DEFAULT_LOG_CONFIG: LogConfig = {
    logLevel: 'Info',
    logDir: 'logs',
    retentionDays: 7,
};

export const DEFAULT_EMAIL_CONFIG: EmailConfig = {
    desktopNotice: true,
    emailNotification: false,
    provider: 'custom',
    smtpServer: '',
    smtpPort: 465,
    security: 'tlsWrapper',
    username: '',
    password: '',
    senderName: 'AutoDaily',
    senderEmail: '',
    recipient: '',
    timeoutSeconds: 60,
};

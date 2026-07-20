import type { AssignmentScheduleStatus } from '@/types/bindings/AssignmentScheduleStatus';
import type { AssignmentTriggerSource } from '@/types/bindings/AssignmentTriggerSource';
import type { ConnectionStatusKind } from '@/types/bindings/ConnectionStatusKind';
import type { DeviceConnectionEventPayload } from '@/types/bindings/DeviceConnectionEventPayload';
import type { EmulatorConnectMode } from '@/types/bindings/EmulatorConnectMode';
import type { DeviceLifecycleStatus } from '@/types/bindings/DeviceLifecycleStatus';
import type { DeviceProgressEventPayload } from '@/types/bindings/DeviceProgressEventPayload';
import type { DeviceRuntimeProgressPhase } from '@/types/bindings/DeviceRuntimeProgressPhase';
import type { DeviceRuntimeReconcileAction as BindingDeviceRuntimeReconcileAction } from '@/types/bindings/DeviceRuntimeReconcileAction';
import type { DeviceRuntimeReconcileEventPayload } from '@/types/bindings/DeviceRuntimeReconcileEventPayload';
import type { DeviceScheduleEventPayload } from '@/types/bindings/DeviceScheduleEventPayload';
import type { DeviceStatusEventPayload } from '@/types/bindings/DeviceStatusEventPayload';
import type { DeviceTimeoutEventPayload } from '@/types/bindings/DeviceTimeoutEventPayload';
import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { DevicePlatform } from '@/types/bindings/DevicePlatform';
import type { DeviceTransportKind } from '@/types/bindings/DeviceTransportKind';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { RunStatus } from '@/types/bindings/RunStatus';
import type { RuntimeType } from '@/types/bindings/RuntimeType';
import type { ScriptVariableCatalog } from '@/types/bindings/ScriptVariableCatalog';
import type { ScriptPlatform } from '@/types/bindings/ScriptPlatform';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type { TimeoutAction } from '@/types/bindings/TimeoutAction';
import type { TimeoutNotifyChannel } from '@/types/bindings/TimeoutNotifyChannel';
import type { JsonValue as StoreJsonValue } from '@/types/bindings/serde_json/JsonValue';
export type { RunTarget } from '@/types/bindings/RunTarget';

export type JsonValue = StoreJsonValue;

export type AppTheme = 'light' | 'dark' | 'system';
export type DefaultRoute = '/' | '/tasks' | '/devices' | '/scripts' | '/market' | '/logs' | '/settings' | '/about' | '/vision-lab';
export type StartMode = 'normal' | 'minimized' | 'tray';
export type IdleAction = 'none' | 'shutdown' | 'sleep' | 'hibernate';
export type DeviceStatusKind = 'idle' | 'running' | 'paused' | 'stopped' | 'error' | 'unknown';
export type DeviceConnectionKind = 'unknown' | 'checking' | 'connected' | 'disconnected';
export type DeviceRuntimeReconcileAction = BindingDeviceRuntimeReconcileAction | null;
export type DeviceRuntimeRawStatus = DeviceLifecycleStatus | ConnectionStatusKind | null;

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
    dispatchScheduleRetentionDays: number;
    shortcut: ShortCutConfig;
}

export interface SystemConfigPayload {
    startMode: StartMode;
    closeExit: boolean;
    alwaysOnTop: boolean;
    idleAction: IdleAction;
    maxIdleRetryNum: number;
    autoStart: boolean;
    dispatchScheduleRetentionDays: number;
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

export interface VisionLabPreferences {
    imageDir: string;
    saveDir: string;
    filterText: string;
}

export interface VisionLabModelConfig {
    imgDetModel: DetectorType | null;
    txtDetModel: DetectorType | null;
    txtRecModel: RecognizerType | null;
}

export interface VisionLabLaunchPreset {
    source: 'editor';
    scriptId: string | null;
    scriptName: string | null;
    selectedDeviceId: string | null;
    imgDetModel: DetectorType | null;
    txtDetModel: DetectorType | null;
    txtRecModel: RecognizerType | null;
    createdAt: string;
}

export interface UserProfile {
    id: string;
    username: string;
    email: string;
    isDeveloper: boolean;
    lastScriptUploadTime: string;
    lastUsernameChangeTime: string;
    sponsorUntil?: string | null;
    authStage?: number | null;
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
    contentMd: string | null;
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
    createTime: string | null;
    updateTime: string | null;
    verName: string;
    scriptType: ScriptType;
    verNum: number;
    latestVer: number;
    downloadCount: number;
    isValid: boolean;
    allowClone: boolean;
    minAppVersion: string | null;
    minRuntimeSchema: number | null;
    requiredFeatures: string[];
    variableCatalog: ScriptVariableCatalog;
    cloudId: string | null;
    runtimeSettings: {
        recoveryTaskId: string | null;
        clickRandomOffset: number;
    };
}

export interface ScriptTableRecord {
    id: string;
    data: ScriptInfoRecord;
}

export interface AssignmentRecord extends Omit<DeviceScriptAssignment, 'accountData'> {
    accountData: JsonValue;
}

export interface DeviceScriptSchedule {
    id: string;
    deviceId: string;
    executionId: string | null;
    assignmentId: string | null;
    scriptId: string;
    taskId: string;
    dedupScopeHash: string;
    taskCycle: string;
    status: RunStatus;
    startedAt: string;
    completedAt: string | null;
    message: string | null;
}

export interface AssignmentSchedule {
    id: string;
    batchId: string;
    deviceId: string;
    assignmentId: string | null;
    scriptId: string | null;
    timeTemplateId: string | null;
    windowStartAt: string | null;
    scopeHash: string;
    dispatchId: string;
    orderIndex: number;
    createdAt: string;
    runTargetJson: string | null;
    status: AssignmentScheduleStatus;
    triggerSource: AssignmentTriggerSource;
    startedAt: string | null;
    completedAt: string | null;
    message: string | null;
}

export interface DeviceRuntimeStatus {
    rawStatus: DeviceRuntimeRawStatus;
    kind: DeviceStatusKind;
    currentScript?: string | null;
    message?: string | null;
}

export interface DeviceStatusEvent {
    deviceId: string;
    sessionId?: string | null;
    status: DeviceLifecycleStatus;
    currentScript?: string | null;
    message?: string | null;
}

export interface DeviceConnectionStatus {
    kind: DeviceConnectionKind;
    rawStatus?: ConnectionStatusKind | null;
    message?: string | null;
    at?: string | null;
}

export interface DeviceRuntimePresence {
    label: string;
    tone: 'neutral' | 'info' | 'success' | 'warning' | 'danger';
    icon: string;
}

export interface DeviceRuntimeProgressView {
    phase: DeviceRuntimeProgressPhase | null;
    label: string;
    tone: 'neutral' | 'info' | 'success' | 'warning' | 'danger';
    message: string | null;
    at: string | null;
}

export interface DeviceRuntimeControlView {
    showStopButton: boolean;
    startLabel: string;
    stopLabel: string;
}

export interface DeviceRuntimeView {
    status: DeviceRuntimeStatus;
    connectionStatus: DeviceConnectionStatus;
    connectionLabel: string;
    connectionTone: 'neutral' | 'info' | 'success' | 'warning' | 'danger';
    presence: DeviceRuntimePresence;
    progress: DeviceRuntimeProgressView;
    pendingMessage: string | null;
    controls: DeviceRuntimeControlView;
}

export interface DeviceRuntimeSnapshot {
    deviceId: string;
    status: DeviceStatusEventPayload | null;
    connection: DeviceConnectionEventPayload | null;
    progress: DeviceProgressEventPayload | null;
}

export type DeviceRuntimeReconcileEvent = DeviceRuntimeReconcileEventPayload;
export type RuntimeLifecycleEvent = DeviceStatusEventPayload;
export type RuntimeProgressEvent = DeviceProgressEventPayload;
export type RuntimeScheduleEvent = DeviceScheduleEventPayload;
export type RuntimeTimeoutEvent = DeviceTimeoutEventPayload;

export interface RuntimeResultProjection {
    deviceId: string;
    latestProgress: RuntimeProgressEvent | null;
    latestSchedule: RuntimeScheduleEvent | null;
    latestTimeout: RuntimeTimeoutEvent | null;
    timeoutActionResult: 'none' | 'pending' | 'skipped' | 'recovered' | 'stopped' | 'failed';
    updatedAt: string | null;
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

export interface ScriptCompatibility {
    compatible: boolean;
    requiredAppVersion: string | null;
    requiredRuntimeSchema: number | null;
    missingFeatures: string[];
    reason: string | null;
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
    minAppVersion?: string | null;
    minRuntimeSchema?: number | null;
    requiredFeatures?: string[] | null;
    compatibility?: ScriptCompatibility | null;
}

export interface ScriptChangeLogRecord {
    id: number | null;
    scriptId: string | null;
    versionName: string | null;
    versionNum: number | null;
    contentMd: string | null;
    createdBy: string | null;
    createdAt: string | null;
    updatedAt: string | null;
}

export interface ScriptCloudSummary {
    id: string | null;
    verName: string | null;
    verNum: number | null;
    latestVer: number | null;
    userName: string | null;
    updateTime: string | null;
}

export type ScriptVersionPreflightStatus =
    | 'noLocalCopy'
    | 'cloudMissing'
    | 'sameVersion'
    | 'upgradeAvailable'
    | 'downgradeBlocked';

export interface ScriptVersionPreflight {
    status: ScriptVersionPreflightStatus;
    message: string;
    localScriptId: string | null;
    localVersionLabel: string | null;
    remoteVersionLabel: string | null;
    localVerNum: number | null;
    remoteVerNum: number | null;
}

export type ScriptUploadActivityStatus = 'waitingAuth' | 'success' | 'error';

export interface ScriptUploadActivity {
    id: string;
    scriptId: string;
    status: ScriptUploadActivityStatus;
    message: string;
    at: string;
    cloudId: string | null;
    username: string | null;
    autoRetry: boolean;
}

export type ScriptTransferDirection = 'upload' | 'download';
export type ScriptTransferStatus = 'running' | 'paused' | 'success' | 'error';

export interface ScriptTransferRecord {
    id: string;
    direction: ScriptTransferDirection;
    localScriptId: string | null;
    cloudScriptId: string | null;
    scriptName: string | null;
    status: ScriptTransferStatus;
    modelFileCount: number;
    completedModelFileCount: number;
    latestFileName: string | null;
    bytesTransferred: number;
    totalBytes: number;
    latestMessage: string | null;
    errorMessage: string | null;
    startedAt: string;
    finishedAt: string | null;
    createdAt: string;
    updatedAt: string;
}

export interface ScriptTransferProgressEvent {
    id: string;
    direction: ScriptTransferDirection;
    localScriptId: string | null;
    cloudScriptId: string | null;
    scriptName: string | null;
    status: ScriptTransferStatus;
    modelFileCount: number;
    completedModelFileCount: number;
    currentFileName: string | null;
    latestFileName: string | null;
    bytesTransferred: number;
    totalBytes: number;
    latestMessage: string | null;
    errorMessage: string | null;
    startedAt: string;
    finishedAt: string | null;
    updatedAt: string;
}

export interface MarketPage<T> {
    records: T[];
    total: number;
    size: number;
    current: number;
}

export interface DeviceFormState {
    id: string | null;
    deviceName: string;
    platform: DevicePlatform;
    transportKind: DeviceTransportKind;
    emulatorConnectMode: EmulatorConnectMode;
    startupDelaySecs: number;
    exePath: string;
    exeArgs: string;
    cores: number[];
    logLevel: LogLevel;
    logToFile: boolean;
    capMethodType: 'window' | 'adb';
    capMethodValue: string;
    windowCaptureInterface: 'dxgi' | 'gdi' | 'dwmGetDxSharedSurface' | 'wgc';
    frameTimeoutSecs: number;
    windowOffsets: string;
    connectAddress: string;
    connectIdentifier: string;
    adbPath: string;
    adbServerConnect: string;
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
    dispatchScheduleRetentionDays: 7,
    shortcut: DEFAULT_SHORTCUTS,
};

export const DEFAULT_LOG_CONFIG: LogConfig = {
    logLevel: 'Info',
    logDir: 'logs',
    retentionDays: 7,
};

export const DEFAULT_VISION_LAB_PREFERENCES: VisionLabPreferences = {
    imageDir: '',
    saveDir: '',
    filterText: '',
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

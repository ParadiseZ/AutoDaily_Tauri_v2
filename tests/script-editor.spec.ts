import { expect, test, type Page } from '@playwright/test';
import type { PolicyGroupTable, PolicySetTable, PolicyTable, ScriptTable, ScriptTaskTable } from '../src/types/bindings';

type StoredScriptTable = Omit<ScriptTable, 'data'> & {
  data: Omit<ScriptTable['data'], 'downloadCount' | 'latestVer' | 'verNum'> & {
    downloadCount: number;
    latestVer: number;
    verNum: number;
  };
};

const emptyVariableCatalog = {
  version: 1,
  variables: [],
} as const;

declare global {
  interface Window {
    __AUTODAILY_MOCK__?: {
      getState: () => {
        scripts: StoredScriptTable[];
        scriptTasks: Record<string, ScriptTaskTable[]>;
        policies: PolicyTable[];
        policyGroups: PolicyGroupTable[];
        policySets: PolicySetTable[];
        groupPolicies: Record<string, string[]>;
        setGroups: Record<string, string[]>;
      };
      reset: () => unknown;
      seed: (partial: {
        scripts?: StoredScriptTable[];
        scriptTasks?: Record<string, ScriptTaskTable[]>;
        policies?: PolicyTable[];
        policyGroups?: PolicyGroupTable[];
        policySets?: PolicySetTable[];
        groupPolicies?: Record<string, string[]>;
        setGroups?: Record<string, string[]>;
      }) => unknown;
    };
  }
}

const seedEditorState = async (page: Page, script: StoredScriptTable) => {
  await page.goto(`/editor?scriptId=${script.id}`);
  await page.evaluate((seedScript) => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.reset();
    window.__AUTODAILY_MOCK__.seed({
      scripts: [seedScript],
      scriptTasks: {},
    });
  }, script);
  await page.reload();
};

const selectOptionByValue = async (page: Page, testId: string, value: string) => {
  const directOption = page.getByTestId(`${testId}-option-${value}`);
  if (await directOption.count()) {
    await directOption.first().evaluate((element: HTMLElement) => element.click());
    return;
  }

  await page.getByTestId(testId).click();
  const dropdownOption = page.getByTestId(`${testId}-option-${value}`);
  await dropdownOption.first().evaluate((element: HTMLElement) => element.click());
};

const selectOptionByLabel = async (page: Page, testId: string, label: string) => {
  const directOption = page.locator(`[data-testid^="${testId}-option-"]`).filter({ hasText: label });
  if (await directOption.count()) {
    await directOption.first().evaluate((element: HTMLElement) => element.click());
    return;
  }

  await page.getByTestId(testId).click();
  const dropdownOption = page.getByTestId(`${testId}-menu`).getByText(label);
  await dropdownOption.first().evaluate((element: HTMLElement) => element.click());
};

const selectEditorMode = async (page: Page, mode: 'task' | 'policy' | 'policyGroup' | 'policySet') => {
  await page.getByTestId(`editor-mode-${mode}`).click();
};

const selectEditorTarget = async (page: Page, id: string) => {
  await selectOptionByValue(page, 'editor-header-target-item', id);
};

const dragStepByHandle = async (page: Page, fromIndex: number, toIndex: number) => {
  const handle = page.getByTestId(`editor-step-drag-${fromIndex}`);
  const target = page.getByTestId(`editor-step-card-${toIndex}`);
  await handle.dispatchEvent('mousedown', { button: 0 });
  await target.dispatchEvent('mouseenter');
  await target.dispatchEvent('mouseup', { button: 0 });
};

const dragRelationByHandle = async (page: Page, fromIndex: number, toIndex: number) => {
  const handle = page.getByTestId(`editor-relation-drag-${fromIndex}`);
  const target = page.getByTestId(`editor-relation-assigned-${toIndex === 0 ? 'policy-a' : 'policy-b'}`);
  await handle.dispatchEvent('mousedown', { button: 0 });
  await target.dispatchEvent('mouseenter');
  await page.evaluate(() => window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, button: 0 })));
};

test('edits script tasks with visual task editor and persists payload', async ({ page }) => {
  const scriptId = 'script-editor-1';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '编辑器验证脚本',
      description: '验证脚本编辑器任务结构保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await expect(page.getByRole('heading', { name: '编辑器验证脚本' })).toBeVisible();
  await expect(page.getByTestId('editor-task-name')).toHaveValue('主任务 1');

  await page.getByTestId('editor-task-name').fill('日常主流程');
  await selectOptionByValue(page, 'editor-task-trigger-mode', 'rootAndLink');
  await selectOptionByValue(page, 'editor-task-tone', 'warning');
  await selectOptionByValue(page, 'editor-task-cycle', 'daily');
  await page.getByTestId('editor-task-hidden').check();
  await page.getByTestId('editor-task-default-enabled').uncheck();

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-key-0').fill('activitySweepCount');
  await page.getByTestId('editor-input-value-0').fill('5');
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-remove-1').click();

  await page.getByTestId('editor-tab-ui').click();
  await page.getByTestId('editor-ui-template-number').click();
  await page.getByTestId('editor-ui-field-label-0').fill('扫荡活动');
  await selectOptionByLabel(page, 'editor-ui-field-bind-0', 'activitySweepCount');
  await page.getByTestId('editor-ui-preview-control-0').fill('8');

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-capture').click();
  await page.getByTestId('editor-step-template-wait').click();

  await expect(page.getByTestId('editor-step-card-0')).toBeVisible();
  await expect(page.getByTestId('editor-step-card-1')).toBeVisible();
  await dragStepByHandle(page, 1, 0);

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scriptTasks[scriptId]).toHaveLength(1);

  const [task] = state!.scriptTasks[scriptId];
  const savedScript = state!.scripts.find((item) => item.id === scriptId);
  expect(task.name).toBe('日常主流程');
  expect(task.rowType).toBe('task');
  expect(task.triggerMode).toBe('rootAndLink');
  expect(task.defaultTaskCycle).toBe('daily');
  expect(task.taskTone).toBe('warning');
  expect(task.defaultEnabled).toBe(false);
  expect(task.isHidden).toBe(true);
  expect(task.data.variables).toEqual({ activitySweepCount: 8 });
  expect(savedScript?.data.variableCatalog.variables).toHaveLength(2);
  expect(savedScript?.data.variableCatalog.variables).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        key: 'input.activitySweepCount',
        name: 'activitySweepCount',
        namespace: 'input',
        valueType: 'int',
        ownerTaskId: task.id,
        persisted: true,
        uiBindable: true,
        defaultValue: 8,
      }),
      expect.objectContaining({
        key: 'runtime.captureResult',
        name: '截图结果',
        namespace: 'runtime',
        valueType: 'image',
        ownerTaskId: task.id,
        persisted: false,
        uiBindable: false,
        defaultValue: null,
      }),
    ]),
  );
  expect(task.data.uiData).toEqual({
    layout: 'horizontal',
    fields: [
      {
        key: 'activitySweepCount',
        label: '扫荡活动',
        control: 'number',
        variableId: savedScript?.data.variableCatalog.variables[0]?.id,
        inputKey: 'activitySweepCount',
      },
    ],
  });
  expect(task.data.steps).toHaveLength(2);
  expect((task.data.steps[0] as { op: string }).op).toBe('flowControl');
  expect((task.data.steps[1] as { op: string }).op).toBe('action');

  await page.reload();
  await expect(page.getByTestId('editor-task-name')).toHaveValue('日常主流程');
  await page.getByTestId('editor-tab-ui').click();
  await expect(page.getByTestId('editor-ui-field-label-0')).toHaveValue('扫荡活动');
  await page.getByTestId('editor-tab-steps').click();
  await expect(page.getByTestId('editor-step-card-1')).toBeVisible();
});

test('persists flow conditions and action forms from step workspace', async ({ page }) => {
  const scriptId = 'script-editor-conditions';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '条件步骤脚本',
      description: '验证条件节点和动作步骤表单保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.condition',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-if').click();
  await page.getByTestId('editor-condition-raw-expr').fill('input.activitySweepCount > 0');

  await page.getByTestId('editor-step-template-click-point').click();
  await page.getByTestId('editor-step-card-1').click();
  await page.getByLabel('X').fill('128');
  await page.getByLabel('Y').fill('256');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps).toHaveLength(2);
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'if',
      con: {
        type: 'rawExpr',
        expr: 'input.activitySweepCount > 0',
      },
    },
  });
  expect(task.data.steps[1]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'point',
      p: {
        x: 128,
        y: 256,
      },
    },
  });
});

test('persists handle policy set flow and policy-set-result condition with ids only', async ({ page }) => {
  const scriptId = 'script-editor-policy-set-flow';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略集流程脚本',
      description: '验证策略集处理步骤与结果条件保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.policy.set.flow',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((seedScript) => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.seed({
      policies: [
        {
          id: 'policy-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '领奖策略',
            note: '策略备注',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
      ],
      policyGroups: [
        {
          id: 'group-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '基础策略组',
            note: '策略组备注',
          },
        },
      ],
      policySets: [
        {
          id: 'set-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '主策略集',
            note: '策略集备注',
          },
        },
      ],
      groupPolicies: {
        'group-a': ['policy-a'],
      },
      setGroups: {
        'set-a': ['group-a'],
      },
    });
  }, script);
  await page.reload();

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-handle-policy-set').click();
  await page.getByTestId('editor-step-template-if').click();

  await page.getByTestId('editor-step-card-0').click();
  await selectOptionByValue(page, 'editor-flow-policy-set-pending', 'set-a');
  await page.getByTestId('editor-flow-policy-set-add').click();
  await expect(page.getByTestId('editor-flow-policy-set-target-set-a')).toContainText('主策略集');

  await page.getByTestId('editor-step-card-1').click();
  await selectOptionByValue(page, 'editor-condition-type', 'policySetResult');
  await selectOptionByValue(page, 'editor-condition-policy-set-result-var', 'runtime.policySetResult');
  await selectOptionByValue(page, 'editor-condition-policy-set-result-field', 'policyId');
  await selectOptionByValue(page, 'editor-condition-policy-set-result-op', 'eq');
  await selectOptionByValue(page, 'editor-condition-policy-set-result-target-id', 'policy-a');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'handlePolicySet',
      target: ['set-a'],
      input_var: 'runtime.policySetImage',
      out_var: 'runtime.policySetResult',
    },
  });
  expect(task.data.steps[1]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'if',
      con: {
        type: 'policySetResult',
        result_var: 'runtime.policySetResult',
        field: 'policyId',
        op: 'eq',
        value_bool: true,
        value_id: 'policy-a',
      },
    },
  });
});

test('loads text-det labels for label actions and persists idx only', async ({ page }) => {
  const scriptId = 'script-editor-label-options';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '标签动作脚本',
      description: '验证标签文件映射与 idx 保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: {
        Yolo11: {
          baseModel: {
            intraThreadNum: 4,
            intraSpinning: true,
            interThreadNum: 1,
            interSpinning: true,
            executionProvider: 'CPU',
            inputWidth: 640,
            inputHeight: 640,
            modelSource: 'BuiltIn',
            modelPath: 'D:\\models\\txt-det.onnx',
            modelType: 'Yolo11',
          },
          classCount: 4,
          confidenceThresh: 0.25,
          iouThresh: 0.45,
          labelPath: 'D:\\models\\txt-det.labels.yaml',
          txtIdx: 0,
        },
      },
      txtRecModel: null,
      pkgName: 'com.example.editor.label-options',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const tasks: ScriptTaskTable[] = [
      {
        id: 'task-label-click',
        scriptId: currentScriptId,
        name: '标签点击任务',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        showEnabledToggle: true,
        defaultEnabled: true,
        taskTone: 'normal',
        isHidden: false,
        data: {
          uiData: {},
          variables: {},
          steps: [
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '点击标签',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'click',
                mode: 'labelIdx',
                idx: 2,
              },
            },
          ],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 0,
      },
    ];

    window.__AUTODAILY_MOCK__?.seed({
      scriptTasks: {
        [currentScriptId]: tasks,
      },
    });
  }, scriptId);
  await page.reload();

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-card-0').click();
  await expect(page.getByTestId('editor-action-click-label-idx')).toContainText('2: 图标');

  await page.getByTestId('editor-action-click-label-idx').click();
  await page.getByTestId('editor-action-click-label-idx-menu').getByText('1: 按钮').click();
  await expect(page.getByTestId('editor-action-click-label-idx')).toContainText('1: 按钮');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'labelIdx',
      idx: 1,
    },
  });
});

test('reminds user to configure text-det label path when label action has no options', async ({ page }) => {
  const scriptId = 'script-editor-label-warning';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '标签提醒脚本',
      description: '验证未配置标签路径时的提示',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.label-warning',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const tasks: ScriptTaskTable[] = [
      {
        id: 'task-label-warning',
        scriptId: currentScriptId,
        name: '标签提醒任务',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        showEnabledToggle: true,
        defaultEnabled: true,
        taskTone: 'normal',
        isHidden: false,
        data: {
          uiData: {},
          variables: {},
          steps: [
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '点击标签',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'click',
                mode: 'labelIdx',
                idx: 0,
              },
            },
          ],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 0,
      },
    ];

    window.__AUTODAILY_MOCK__?.seed({
      scriptTasks: {
        [currentScriptId]: tasks,
      },
    });
  }, scriptId);
  await page.reload();

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-card-0').click();
  await expect(page.getByText('当前脚本未设置文字检测模型的标签文件')).toBeVisible();
});

test('renders script-level task preview with title groups and task metadata', async ({ page }) => {
  const scriptId = 'script-editor-preview';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '任务整表预览脚本',
      description: '验证标题行、分组和任务元数据预览',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.preview',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const previewTitleId = 'task-title-daily';
    const tasks: ScriptTaskTable[] = [
      {
        id: previewTitleId,
        scriptId: currentScriptId,
        name: '每日任务',
        rowType: 'title',
        triggerMode: 'rootOnly',
        recordSchedule: false,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        showEnabledToggle: false,
        defaultEnabled: true,
        taskTone: 'normal',
        isHidden: false,
        data: {
          uiData: {},
          variables: {},
          steps: [],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 0,
      },
      {
        id: 'task-daily-sign',
        scriptId: currentScriptId,
        name: '签到',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: previewTitleId,
        indentLevel: 1,
        defaultTaskCycle: 'daily',
        showEnabledToggle: true,
        defaultEnabled: true,
        taskTone: 'warning',
        isHidden: false,
        data: {
          uiData: {
            layout: 'horizontal',
            fields: [
              {
                key: 'signCount',
                label: '签到次数',
                control: 'number',
                inputKey: 'signCount',
              },
            ],
          },
          variables: {},
          steps: [],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 1,
      },
      {
        id: 'task-loose-reward',
        scriptId: currentScriptId,
        name: '奖励领取',
        rowType: 'task',
        triggerMode: 'rootAndLink',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'weekly',
        showEnabledToggle: true,
        defaultEnabled: false,
        taskTone: 'danger',
        isHidden: false,
        data: {
          uiData: {},
          variables: {},
          steps: [],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 2,
      },
    ];

    window.__AUTODAILY_MOCK__?.seed({
      scriptTasks: {
        [currentScriptId]: tasks,
      },
    });
  }, scriptId);
  await page.reload();

  await selectEditorTarget(page, 'task-daily-sign');
  await page.getByTestId('editor-tab-ui').click();

  await expect(page.getByText('整表任务预览')).toBeVisible();
  await expect(page.getByText('每日任务').first()).toBeVisible();
  await expect(page.getByText('签到').first()).toBeVisible();
  await expect(page.getByText('奖励领取').first()).toBeVisible();
  await expect(page.getByText('未分组任务')).toBeVisible();
  await expect(page.getByText('每日').first()).toBeVisible();
  await expect(page.getByRole('heading', { name: 'UI 预览/签到' })).toBeVisible();
});

test('persists varCompare conditions and nested branch steps', async ({ page }) => {
  const scriptId = 'script-editor-nested';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '嵌套分支脚本',
      description: '验证 varCompare 和嵌套步骤保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.nested',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-key-0').fill('pkgName');
  await selectOptionByValue(page, 'editor-input-type-0', 'string');

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-if').click();

  await selectOptionByValue(page, 'editor-condition-type', 'varCompare');
  await selectOptionByValue(page, 'editor-condition-var-name', 'input.pkgName');
  await page.getByLabel('比较值').fill('已完成');
  await page.getByTestId('editor-branch-then').click();
  await page.getByTestId('editor-step-template-wait').click();
  await page.getByRole('button', { name: '顶层步骤' }).click();
  await expect(page.getByTestId('editor-step-card-0')).toBeVisible();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByTestId('editor-branch-then').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'if',
      con: {
        type: 'varCompare',
        var_name: 'input.pkgName',
        value: '已完成',
      },
      then: [
        {
          op: 'flowControl',
          a: {
            type: 'waitMs',
          },
        },
      ],
    },
  });
});

test('persists sequence, vision rule, and task state forms', async ({ page }) => {
  const scriptId = 'script-editor-leaf-forms';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '叶子表单脚本',
      description: '验证 sequence、vision、taskControl 表单保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.leaf',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const tasks: ScriptTaskTable[] = [
      {
        id: 'daily_task',
        scriptId: currentScriptId,
        name: '日常任务',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        showEnabledToggle: true,
        defaultEnabled: true,
        taskTone: 'normal',
        isHidden: false,
        data: {
          uiData: {},
          variables: {},
          steps: [],
        },
        createdAt: '2026-03-26T08:00:00.000Z',
        updatedAt: '2026-03-26T08:00:00.000Z',
        deletedAt: null,
        isDeleted: false,
        index: 0,
      },
    ];

    window.__AUTODAILY_MOCK__?.seed({
      scriptTasks: {
        [currentScriptId]: tasks,
      },
    });
  }, scriptId);
  await page.reload();

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-key-0').fill('sweepLimit');
  await selectOptionByValue(page, 'editor-input-type-0', 'float');
  await page.getByTestId('editor-input-add').click();
  await page.getByLabel('键').fill('retryCount');

  await page.getByTestId('editor-tab-steps').click();

  await page.getByTestId('editor-step-template-sequence').click();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByLabel('倒序执行子步骤').check();

  await page.getByTestId('editor-step-template-vision-search').click();
  await page.getByTestId('editor-step-card-1').click();
  await page.getByRole('button', { name: '添加文本' }).click();
  await page.getByTestId('editor-search-rule-item-0-txt').fill('领取');

  await page.getByTestId('editor-step-template-set-task-state').click();
  await page.getByTestId('editor-step-card-2').click();
  await selectOptionByValue(page, 'editor-task-control-target', 'daily_task');
  await page.getByLabel('状态值为真').uncheck();

  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-3').click();
  await selectOptionByLabel(page, 'editor-set-var-name', 'sweepLimit');
  await page.getByTestId('editor-set-var-value').fill('1.5');

  await page.getByTestId('editor-step-template-get-var').click();
  await page.getByTestId('editor-step-card-4').click();
  await selectOptionByLabel(page, 'editor-get-var-name', 'retryCount');
  await page.getByLabel('启用默认值').check();
  await selectOptionByValue(page, 'editor-get-var-type', 'int');
  await page.getByTestId('editor-get-var-value').fill('3');

  await page.getByTestId('editor-step-template-filter-var').click();
  await page.getByTestId('editor-step-card-5').click();
  await page.getByTestId('editor-branch-filterThen').click();
  await page.getByTestId('editor-step-template-click-text').click();
  await page.getByLabel('目标文字').fill('命中');
  await page.getByTestId('editor-step-template-wait').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'sequence',
    reverse: true,
  });
  expect(task.data.steps[1]).toMatchObject({
    op: 'vision',
    a: {
      type: 'visionSearch',
      out_var: 'runtime.visionHit',
      rule: {
        type: 'group',
        op: 'And',
        scope: 'Global',
        items: [
          {
            type: 'txt',
            pattern: '领取',
          },
        ],
      },
    },
  });
  expect(task.data.steps[2]).toMatchObject({
    op: 'taskControl',
    a: {
      type: 'setState',
      target: {
        type: 'task',
        id: 'daily_task',
      },
      status: {
        type: 'done',
        value: false,
      },
    },
  });
  expect(task.data.steps[3]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'setVar',
      name: 'input.sweepLimit',
      val: 1.5,
      expr: null,
    },
  });
  expect(task.data.steps[4]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'getVar',
      name: 'input.retryCount',
      default_val: 3,
    },
  });
  expect(task.data.steps[5]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'filter',
      then_steps: [
        {
          op: 'action',
          a: {
            ac: 'click',
            mode: 'txt',
            txt: '命中',
          },
        },
        {
          op: 'flowControl',
          a: {
            type: 'waitMs',
          },
        },
      ],
    },
  });
});

test('creates variable from setVar template and persists catalog binding', async ({ page }) => {
  const scriptId = 'script-editor-setvar-create';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '变量创建脚本',
      description: '验证 setVar 内联创建变量并保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.setvar.create',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByTestId('editor-set-var-value').fill('7');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const savedScript = state!.scripts.find((item) => item.id === scriptId);
  const [task] = state!.scriptTasks[scriptId];

  expect(savedScript?.data.variableCatalog.variables).toHaveLength(1);
  expect(savedScript?.data.variableCatalog.variables[0]).toMatchObject({
    key: 'input.newVar1',
    name: 'newVar1',
    namespace: 'input',
    valueType: 'int',
  });
  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'setVar',
      name: 'input.newVar1',
      val: 7,
      expr: null,
    },
  });
});

test('renames input variable and syncs setVar reference', async ({ page }) => {
  const scriptId = 'script-editor-setvar-rename';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '变量重命名脚本',
      description: '验证输入变量重命名后同步步骤引用',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.setvar.rename',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByTestId('editor-set-var-locate').click();
  await page.getByTestId('editor-input-key-0').fill('renamedVar');
  await page.getByTestId('editor-tab-steps').click();
  await expect(page.getByTestId('editor-set-var-name')).toContainText('renamedVar');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'setVar',
      name: 'input.renamedVar',
    },
  });
});

test('shows variable key in setVar selector when variable name differs', async ({ page }) => {
  const scriptId = 'script-editor-setvar-option-copy';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '变量选项文案脚本',
      description: '验证 setVar 变量选择展示名称和键',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.setvar.option-copy',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByLabel('名称').fill('扫荡次数');
  await page.getByTestId('editor-input-key-0').fill('sweepLimit');
  await selectOptionByValue(page, 'editor-input-type-0', 'int');

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();
  await expect(page.locator('[data-testid^="editor-set-var-name-option-"]').filter({ hasText: 'sweepLimit' })).toBeVisible();
  await expect(page.locator('[data-testid^="editor-set-var-name-option-"]').filter({ hasText: 'Input · 整数' }).first()).toBeVisible();
});

test('shows incomplete draft input in setVar selector as disabled option', async ({ page }) => {
  const scriptId = 'script-editor-setvar-draft-option';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '变量草稿脚本',
      description: '验证步骤变量选择包含未完成输入草稿',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.setvar.draft-option',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByLabel('名称').fill('test');
  await page.getByTestId('editor-input-key-0').fill('newVar1');
  await page.getByTestId('editor-input-add').click();

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();

  const variableOptions = page.locator('[data-testid^="editor-set-var-name-option-"]');
  await expect(variableOptions).toHaveCount(3);
  await expect(variableOptions.filter({ hasText: '未设置键' })).toBeVisible();
  await expect(variableOptions.filter({ hasText: '需先填写键' })).toBeVisible();
});

test('registers filter template variables and persists bindings', async ({ page }) => {
  const scriptId = 'script-editor-filter-selectors';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '过滤变量模板脚本',
      description: '验证 filter 模板会自动登记输入和输出变量',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.filter.selectors',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-filter-var').click();
  await page.getByTestId('editor-step-card-0').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  const savedScript = state!.scripts.find((item) => item.id === scriptId);

  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'filter',
      input_var: 'input.items',
      out_name: 'runtime.filteredItems',
    },
  });
  expect(savedScript?.data.variableCatalog.variables).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        key: 'input.items',
        namespace: 'input',
        valueType: 'json',
      }),
      expect.objectContaining({
        key: 'runtime.filteredItems',
        namespace: 'runtime',
        valueType: 'json',
      }),
    ]),
  );
});

test('registers capture template variable and supports inline editing from step workspace', async ({ page }) => {
  const scriptId = 'script-editor-capture-inline-variable';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '截图变量脚本',
      description: '验证 capture 模板自动登记变量并可在步骤区直接编辑',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.capture.inline-variable',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-capture').click();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByPlaceholder('例如：扫荡次数').fill('截图载荷');
  await page.getByPlaceholder('例如：activitySweepCount').fill('capturePayload');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const savedScript = state!.scripts.find((item) => item.id === scriptId);
  const [task] = state!.scriptTasks[scriptId];

  expect(task.data.steps[0]).toMatchObject({
    op: 'action',
    a: {
      ac: 'capture',
      output_var: 'runtime.capturePayload',
    },
  });
  expect(savedScript?.data.variableCatalog.variables).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        key: 'runtime.capturePayload',
        name: '截图载荷',
        namespace: 'runtime',
        valueType: 'image',
      }),
    ]),
  );
});

test('removes auto-created variables when deleting the source step', async ({ page }) => {
  const scriptId = 'script-editor-remove-step-variable';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '删除步骤变量脚本',
      description: '验证删除自动创建变量来源步骤时会回收变量',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.remove-step-variable',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-capture').click();
  await expect(page.getByTestId('editor-step-card-0')).toBeVisible();
  await page.getByTestId('editor-step-card-0').getByRole('button', { name: '删除' }).click();
  await expect(page.getByTestId('editor-step-card-0')).toHaveCount(0);

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const savedScript = state!.scripts.find((item) => item.id === scriptId);
  const [task] = state!.scriptTasks[scriptId];

  expect(task.data.steps).toEqual([]);
  expect(savedScript?.data.variableCatalog.variables).toEqual([]);
});

test('registers runtime output from vision template', async ({ page }) => {
  const scriptId = 'script-editor-vision-runtime-output';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '视觉输出变量脚本',
      description: '验证 vision 模板自动登记 runtime 输出变量',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.vision.runtime-output',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-vision-search').click();
  await page.getByTestId('editor-step-card-0').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const savedScript = state!.scripts.find((item) => item.id === scriptId);
  const [task] = state!.scriptTasks[scriptId];

  expect(task.data.steps[0]).toMatchObject({
    op: 'vision',
    a: {
      type: 'visionSearch',
      out_var: 'runtime.visionHit',
    },
  });
  expect(savedScript?.data.variableCatalog.variables).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        key: 'runtime.visionHit',
        namespace: 'runtime',
        valueType: 'json',
      }),
    ]),
  );
});

test('creates policies and persists search rule with before and after actions', async ({ page }) => {
  const scriptId = 'script-editor-policy';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略编辑脚本',
      description: '验证策略编辑链路保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.policy',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await selectEditorMode(page, 'policy');
  await page.getByTestId('editor-policy-create').click();
  await page.getByTestId('editor-policy-name').fill('领奖策略');

  await page.getByTestId('editor-policy-tab-condition').click();
  await page.getByRole('button', { name: '添加文本' }).click();
  await page.getByTestId('editor-policy-condition-item-0-txt').fill('领取');

  await page.getByTestId('editor-policy-tab-before').click();
  await page.getByTestId('editor-policy-step-template-wait').click();

  await page.getByTestId('editor-policy-tab-after').click();
  await page.getByTestId('editor-policy-step-template-click-text').click();
  await page.getByLabel('目标文字').fill('领取');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.policies).toHaveLength(1);
  expect(state?.policies[0]).toMatchObject({
    scriptId,
    orderIndex: 0,
    data: {
      name: '领奖策略',
      beforeAction: [
        {
          op: 'flowControl',
          a: {
            type: 'waitMs',
          },
        },
      ],
      cond: {
        type: 'group',
        op: 'And',
        scope: 'Global',
        items: [
          {
            type: 'txt',
            pattern: '领取',
          },
        ],
      },
      afterAction: [
        {
          op: 'action',
          a: {
            ac: 'click',
            mode: 'txt',
            txt: '领取',
          },
        },
      ],
    },
  });
});

test('persists policyCondition with relative rule in task flow', async ({ page }) => {
  const scriptId = 'script-editor-policy-condition-relative';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略条件相对位置脚本',
      description: '验证 PolicyCondition 的相对位置规则保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.policy-condition.relative',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-if').click();
  await selectOptionByValue(page, 'editor-condition-type', 'policyCondition');
  await selectOptionByValue(page, 'editor-condition-policy-condition-rule-type', 'relative');
  await selectOptionByValue(page, 'editor-condition-policy-condition-rule-relative-direction', 'right');
  await selectOptionByValue(page, 'editor-condition-policy-condition-rule-relative-target-kind', 'ocrText');
  await selectOptionByValue(page, 'editor-condition-policy-condition-rule-relative-value-type', 'number');
  await selectOptionByValue(page, 'editor-condition-policy-condition-rule-relative-compare', 'gt');
  await page.getByTestId('editor-condition-policy-condition-rule-relative-anchor-text').fill('结晶');
  await page.getByTestId('editor-condition-policy-condition-rule-relative-value').fill('5');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'if',
      con: {
        type: 'policyCondition',
        input_var: null,
        rule: {
          type: 'relative',
          anchor_type: 'ocrText',
          anchor_text: '结晶',
          anchor_idx: 0,
          direction: 'right',
          target_kind: 'ocrText',
          value_type: 'number',
          compare: 'gt',
          value: '5',
        },
      },
    },
  });
});

test('reorders assigned policies inside policy group workspace', async ({ page }) => {
  const scriptId = 'script-editor-policy-group';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略组排序脚本',
      description: '验证策略组关联排序',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.policy.group',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await page.goto(`/editor?scriptId=${script.id}`);
  await page.evaluate((seedScript) => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.reset();
    window.__AUTODAILY_MOCK__.seed({
      scripts: [seedScript],
      scriptTasks: {},
      policies: [
        {
          id: 'policy-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '登录',
            note: '测试备注A',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
        {
          id: 'policy-b',
          scriptId: seedScript.id,
          orderIndex: 1,
          data: {
            name: '领奖',
            note: '测试备注B',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
      ],
      policyGroups: [
        {
          id: 'group-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '基础策略组',
            note: '测试策略组',
          },
        },
      ],
      policySets: [],
      groupPolicies: {
        'group-a': ['policy-a', 'policy-b'],
      },
      setGroups: {},
    });
  }, script);
  await page.reload();

  await selectEditorMode(page, 'policyGroup');
  await dragRelationByHandle(page, 1, 0);
  await expect(page.getByTestId('editor-relation-assigned-policy-b')).toContainText('1');
  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.groupPolicies['group-a']).toEqual(['policy-b', 'policy-a']);
});

test('reverses assigned policies inside policy group workspace', async ({ page }) => {
  const scriptId = 'script-editor-policy-group-reverse';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略组逆序脚本',
      description: '验证策略组关联逆序按钮',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      pkgName: 'com.example.editor.policy.group.reverse',
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: emptyVariableCatalog,
      cloudId: null,
    },
  };

  await page.goto(`/editor?scriptId=${script.id}`);
  await page.evaluate((seedScript) => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.reset();
    window.__AUTODAILY_MOCK__.seed({
      scripts: [seedScript],
      scriptTasks: {},
      policies: [
        {
          id: 'policy-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '登录',
            note: '测试备注A',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
        {
          id: 'policy-b',
          scriptId: seedScript.id,
          orderIndex: 1,
          data: {
            name: '领奖',
            note: '测试备注B',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
        {
          id: 'policy-c',
          scriptId: seedScript.id,
          orderIndex: 2,
          data: {
            name: '收尾',
            note: '测试备注C',
            logPrint: null,
            curPos: 0,
            skipFlag: false,
            execCur: 0,
            execMax: 1,
            beforeAction: [],
            cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
            afterAction: [],
          },
        },
      ],
      policyGroups: [
        {
          id: 'group-a',
          scriptId: seedScript.id,
          orderIndex: 0,
          data: {
            name: '基础策略组',
            note: '测试策略组',
          },
        },
      ],
      policySets: [],
      groupPolicies: {
        'group-a': ['policy-a', 'policy-b', 'policy-c'],
      },
      setGroups: {},
    });
  }, script);
  await page.reload();

  await selectEditorMode(page, 'policyGroup');
  await page.getByTestId('editor-relation-reverse').evaluate((element: HTMLElement) => element.click());
  await expect(page.getByTestId('editor-relation-assigned-policy-c')).toContainText('1');
  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.groupPolicies['group-a']).toEqual(['policy-c', 'policy-b', 'policy-a']);
});

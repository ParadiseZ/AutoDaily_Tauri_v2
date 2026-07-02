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

const fillCodeEditor = async (page: Page, testId: string, value: string) => {
  const editor = page.locator(`[data-testid="${testId}"] .cm-content`).first();
  await editor.click();
  await editor.press('Control+A');
  await page.keyboard.press('Backspace');
  await page.keyboard.insertText(value);
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

const openTaskContextMenu = async (page: Page, taskId: string) => {
  await page.getByTestId(`editor-task-item-${taskId}`).dispatchEvent('contextmenu', {
    button: 2,
    clientX: 240,
    clientY: 240,
  });
  await expect(page.getByTestId('editor-task-context-menu')).toBeVisible();
};

const openTaskContextMenuWithShift = async (page: Page, taskId: string) => {
  await page.getByTestId(`editor-task-item-${taskId}`).evaluate((element: HTMLElement) => {
    element.dispatchEvent(new MouseEvent('contextmenu', {
      bubbles: true,
      cancelable: true,
      button: 2,
      shiftKey: true,
      clientX: 240,
      clientY: 240,
    }));
  });
};

const openCollectionContextMenu = async (page: Page, prefix: string, itemId: string) => {
  await page.getByTestId(`${prefix}-item-${itemId}`).dispatchEvent('contextmenu', {
    button: 2,
    clientX: 240,
    clientY: 240,
  });
  await expect(page.getByTestId(`${prefix}-context-menu`)).toBeVisible();
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
  await page.getByTestId('editor-input-value-0').fill('8');
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-remove-1').click();

  await page.getByTestId('editor-tab-ui').click();
  await page.getByTestId('editor-ui-template-number').click();
  await page.getByTestId('editor-ui-field-label-0').fill('扫荡活动');
  await selectOptionByLabel(page, 'editor-ui-field-bind-0', 'activitySweepCount');

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

test('persists task description from basic config panel', async ({ page }) => {
  const scriptId = 'script-editor-task-description';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '任务说明保存脚本',
      description: '验证任务说明字段保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'input-counter',
            key: 'input.counter',
            name: '计数器',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 3,
            description: 'Input · 整数',
          },
          {
            id: 'runtime-items',
            key: 'runtime.items',
            name: '结果集',
            namespace: 'runtime',
            valueType: 'list',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: 'Runtime · 列表',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-task-description').fill('用于说明任务执行前置条件');
  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scriptTasks[scriptId]?.[0]?.description).toBe('用于说明任务执行前置条件');
});

test('allows clearing task and title row names without auto-filling text', async ({ page }) => {
  const scriptId = 'script-editor-empty-name';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '空名称验证脚本',
      description: '验证任务名称删除后不会自动回填',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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

  const taskNameInput = page.getByTestId('editor-task-name');
  await expect(taskNameInput).toHaveValue('主任务 1');

  await taskNameInput.fill('');
  await expect(taskNameInput).toHaveValue('');

  await page.getByTestId('editor-save').click();

  let state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scriptTasks[scriptId]?.[0]?.name).toBe('');

  await selectOptionByValue(page, 'editor-task-row-type', 'title');
  await expect(taskNameInput).toHaveValue('');

  await taskNameInput.fill('分组标题');
  await expect(taskNameInput).toHaveValue('分组标题');
  await taskNameInput.fill('');
  await expect(taskNameInput).toHaveValue('');

  await page.getByTestId('editor-save').click();

  state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scriptTasks[scriptId]?.[0]?.rowType).toBe('title');
  expect(state?.scriptTasks[scriptId]?.[0]?.name).toBe('');

  await page.reload();
  await expect(page.getByTestId('editor-task-name')).toHaveValue('');
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
  await page.getByRole('spinbutton', { name: 'X', exact: true }).fill('128');
  await page.getByRole('spinbutton', { name: 'Y', exact: true }).fill('256');

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
  await expect(page.getByTestId('editor-flow-policy-set-det-input-var')).toContainText('检测结果');
  await expect(page.getByTestId('editor-flow-policy-set-ocr-input-var')).toContainText('OCR结果');
  await expect(page.getByTestId('editor-flow-policy-set-search-hits-var')).toContainText('搜索命中');

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
      det_input_var: 'runtime.detResults',
      ocr_input_var: 'runtime.ocrResults',
      search_hits_var: 'runtime.searchHits',
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

test('persists policy binding flow steps with top and reverse flags', async ({ page }) => {
  const scriptId = 'script-editor-policy-bindings';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略绑定脚本',
      description: '验证策略绑定、追加与移除步骤保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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
            name: '登录策略',
            note: '策略备注 A',
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
            name: '领奖策略',
            note: '策略备注 B',
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
            note: '策略组备注 A',
          },
        },
        {
          id: 'group-b',
          scriptId: seedScript.id,
          orderIndex: 1,
          data: {
            name: '扩展策略组',
            note: '策略组备注 B',
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
            note: '策略集备注 A',
          },
        },
        {
          id: 'set-b',
          scriptId: seedScript.id,
          orderIndex: 1,
          data: {
            name: '备用策略集',
            note: '策略集备注 B',
          },
        },
      ],
      groupPolicies: {
        'group-a': ['policy-a'],
        'group-b': ['policy-b'],
      },
      setGroups: {
        'set-a': ['group-a'],
        'set-b': ['group-b'],
      },
    });
  }, script);
  await page.reload();

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-add-policies').click();
  await page.getByTestId('editor-step-template-remove-policies').click();
  await page.getByTestId('editor-step-template-bind-policy-group').click();
  await page.getByTestId('editor-step-template-remove-policy-group').click();
  await page.getByTestId('editor-step-template-add-policy-groups').click();
  await page.getByTestId('editor-step-template-unload-policy-group').click();
  await page.getByTestId('editor-step-template-bind-policy').click();
  await page.getByTestId('editor-step-template-unload-policy').click();

  await page.getByTestId('editor-step-card-0').click();
  await selectOptionByValue(page, 'editor-flow-add-policies-source', 'set-a');
  await selectOptionByValue(page, 'editor-flow-add-policies-target', 'set-b');
  await page.getByTestId('editor-flow-add-policies-top').check();
  await page.getByTestId('editor-flow-add-policies-reverse').check();

  await page.getByTestId('editor-step-card-1').click();
  await selectOptionByValue(page, 'editor-flow-remove-policies-source', 'set-a');
  await selectOptionByValue(page, 'editor-flow-remove-policies-target', 'set-b');

  await page.getByTestId('editor-step-card-2').click();
  await selectOptionByValue(page, 'editor-flow-bind-policy-group-source', 'group-a');
  await selectOptionByValue(page, 'editor-flow-bind-policy-group-target', 'set-b');
  await page.getByTestId('editor-flow-bind-policy-group-top').check();

  await page.getByTestId('editor-step-card-3').click();
  await selectOptionByValue(page, 'editor-flow-remove-policy-group-source', 'group-a');
  await selectOptionByValue(page, 'editor-flow-remove-policy-group-target', 'set-b');

  await page.getByTestId('editor-step-card-4').click();
  await selectOptionByValue(page, 'editor-flow-add-policy-groups-source', 'group-b');
  await selectOptionByValue(page, 'editor-flow-add-policy-groups-target', 'group-a');
  await page.getByTestId('editor-flow-add-policy-groups-top').check();
  await page.getByTestId('editor-flow-add-policy-groups-reverse').check();

  await page.getByTestId('editor-step-card-5').click();
  await selectOptionByValue(page, 'editor-flow-unload-policy-group-source', 'group-b');
  await selectOptionByValue(page, 'editor-flow-unload-policy-group-target', 'group-a');

  await page.getByTestId('editor-step-card-6').click();
  await selectOptionByValue(page, 'editor-flow-bind-policy-source', 'policy-b');
  await selectOptionByValue(page, 'editor-flow-bind-policy-target', 'group-a');
  await page.getByTestId('editor-flow-bind-policy-reverse').check();

  await page.getByTestId('editor-step-card-7').click();
  await selectOptionByValue(page, 'editor-flow-unload-policy-source', 'policy-b');
  await selectOptionByValue(page, 'editor-flow-unload-policy-target', 'group-a');

  await expect(page.getByTestId('editor-step-card-0')).toContainText('主策略集 -> 备用策略集');
  await expect(page.getByTestId('editor-step-card-1')).toContainText('移除策略集 主策略集 -> 备用策略集');
  await expect(page.getByTestId('editor-step-card-2')).toContainText('基础策略组 -> 策略集 备用策略集');
  await expect(page.getByTestId('editor-step-card-3')).toContainText('移除策略组 基础策略组 -> 策略集 备用策略集');
  await expect(page.getByTestId('editor-step-card-4')).toContainText('扩展策略组 -> 策略组 基础策略组');
  await expect(page.getByTestId('editor-step-card-5')).toContainText('卸载策略组 扩展策略组 -> 策略组 基础策略组');
  await expect(page.getByTestId('editor-step-card-6')).toContainText('领奖策略 -> 策略组 基础策略组');
  await expect(page.getByTestId('editor-step-card-7')).toContainText('卸载策略 领奖策略 -> 策略组 基础策略组');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'addPolicies',
      source: 'set-a',
      target: 'set-b',
      top: true,
      reverse: true,
    },
  });
  expect(task.data.steps[1]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'removePolicies',
      source: 'set-a',
      target: 'set-b',
    },
  });
  expect(task.data.steps[2]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'bindPolicyGroup',
      source: 'group-a',
      target: 'set-b',
      top: true,
      reverse: false,
    },
  });
  expect(task.data.steps[3]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'removePolicyGroup',
      source: 'group-a',
      target: 'set-b',
    },
  });
  expect(task.data.steps[4]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'addPolicyGroups',
      source: 'group-b',
      target: 'group-a',
      top: true,
      reverse: true,
    },
  });
  expect(task.data.steps[5]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'unloadPolicyGroup',
      source: 'group-b',
      target: 'group-a',
    },
  });
  expect(task.data.steps[6]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'bindPolicy',
      source: 'policy-b',
      target: 'group-a',
      top: false,
      reverse: true,
    },
  });
  expect(task.data.steps[7]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'unloadPolicy',
      source: 'policy-b',
      target: 'group-a',
    },
  });
});

test('loads img-det labels for label actions and persists idx only', async ({ page }) => {
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
      imgDetModel: {
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
            modelPath: 'D:\\models\\img-det.onnx',
            modelType: 'Yolo11',
          },
          classCount: 4,
          confidenceThresh: 0.25,
          iouThresh: 0.45,
          labelPath: 'D:\\models\\img-det.labels.yaml',
          txtIdx: 0,
          postprocessKind: 'LegacyNms',
        },
      },
      txtDetModel: null,
      txtRecModel: null,
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

test('reminds user to configure img-det label path when label action has no options', async ({ page }) => {
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
  await expect(page.getByText('当前脚本未设置图像检测模型的标签文件')).toBeVisible();
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

  await page.getByTestId('editor-task-item-task-daily-sign').getByRole('button', { name: /签到/ }).evaluate((element: HTMLElement) => element.click());
  await page.getByTestId('editor-tab-ui').click();
  await page.getByRole('button', { name: 'UI 预览' }).click();

  await expect(page.getByText('整表任务预览')).toBeVisible();
  await expect(page.getByText('每日任务').first()).toBeVisible();
  await expect(page.getByText('签到').first()).toBeVisible();
  await expect(page.getByText('奖励领取').first()).toBeVisible();
  await expect(page.getByText('未分组任务')).toBeVisible();
  await expect(page.getByText('每日').first()).toBeVisible();
  await expect(page.locator('.editor-ui-task-name', { hasText: '签到' })).toBeVisible();
});

test('renders task description on second line in ui preview', async ({ page }) => {
  const scriptId = 'script-editor-preview-description';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '任务说明预览脚本',
      description: '验证预览中的任务说明',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'input-counter',
            key: 'input.counter',
            name: '计数器',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 3,
            description: 'Input · 整数',
          },
          {
            id: 'runtime-items',
            key: 'runtime.items',
            name: '结果集',
            namespace: 'runtime',
            valueType: 'list',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: 'Runtime · 列表',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const tasks: ScriptTaskTable[] = [
      {
        id: 'task-preview-description',
        scriptId: currentScriptId,
        name: '体力领取',
        description: '午间和晚间各执行一次',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'daily',
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

  await page.getByTestId('editor-tab-ui').click();
  await page.getByRole('button', { name: 'UI 预览' }).click();

  await expect(page.locator('.editor-ui-task-name', { hasText: '体力领取' })).toBeVisible();
  await expect(page.locator('.editor-ui-task-description', { hasText: '午间和晚间各执行一次' })).toBeVisible();
});

test('collapses task groups and moves tasks from context menu', async ({ page }) => {
  const scriptId = 'script-editor-task-context-move';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '任务分组菜单脚本',
      description: '验证左栏折叠和右键移动菜单',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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
    const titleGlobalId = 'title-global-settings';
    const titleDailyId = 'title-daily';
    const tasks: ScriptTaskTable[] = [
      {
        id: titleGlobalId,
        scriptId: currentScriptId,
        name: '全局设置',
        rowType: 'title',
        triggerMode: 'rootOnly',
        recordSchedule: false,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        execMax: 0,
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
        id: 'task-auto-download',
        scriptId: currentScriptId,
        name: '自动下载资源',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: titleGlobalId,
        indentLevel: 1,
        defaultTaskCycle: 'daily',
        execMax: 0,
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
        index: 1,
      },
      {
        id: titleDailyId,
        scriptId: currentScriptId,
        name: '每日任务',
        rowType: 'title',
        triggerMode: 'rootOnly',
        recordSchedule: false,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'everyRun',
        execMax: 0,
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
        index: 2,
      },
      {
        id: 'task-daily-sign',
        scriptId: currentScriptId,
        name: '签到',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: titleDailyId,
        indentLevel: 1,
        defaultTaskCycle: 'daily',
        execMax: 0,
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
        index: 3,
      },
      {
        id: 'task-receive-reward',
        scriptId: currentScriptId,
        name: '奖励领取',
        rowType: 'task',
        triggerMode: 'rootOnly',
        recordSchedule: true,
        sectionId: null,
        indentLevel: 0,
        defaultTaskCycle: 'weekly',
        execMax: 0,
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
        index: 4,
      },
    ];

    window.__AUTODAILY_MOCK__?.seed({
      scriptTasks: {
        [currentScriptId]: tasks,
      },
    });
  }, scriptId);
  await page.reload();

  await expect(page.getByTestId('editor-task-item-task-auto-download')).toBeVisible();
  await page.getByTestId('editor-task-group-toggle-title-global-settings').click();
  await expect(page.getByTestId('editor-task-item-task-auto-download')).toBeHidden();
  await page.getByTestId('editor-task-group-toggle-title-global-settings').click();
  await expect(page.getByTestId('editor-task-item-task-auto-download')).toBeVisible();

  await openTaskContextMenu(page, 'title-global-settings');
  await expect(page.getByTestId('editor-task-context-menu')).toBeVisible();

  await openTaskContextMenuWithShift(page, 'task-daily-sign');
  await expect(page.getByTestId('editor-task-context-menu')).toHaveCount(0);

  await openTaskContextMenu(page, 'task-daily-sign');
  await page.getByTestId('editor-task-move-section').dispatchEvent('mouseenter');
  await page.getByTestId('editor-task-move-section-item-title-global-settings').dispatchEvent('mouseenter');
  await page.getByTestId('editor-task-move-section-title-global-settings-top').evaluate((element: HTMLElement) => element.click());

  await openTaskContextMenu(page, 'task-daily-sign');
  await page.getByTestId('editor-task-move-current-bottom').evaluate((element: HTMLElement) => element.click());

  await openTaskContextMenu(page, 'task-receive-reward');
  await page.getByTestId('editor-task-move-task').dispatchEvent('mouseenter');
  await page.getByTestId('editor-task-move-task-item-task-auto-download').dispatchEvent('mouseenter');
  await page.getByTestId('editor-task-move-task-task-auto-download-bottom').evaluate((element: HTMLElement) => element.click());

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scriptTasks[scriptId].map((task) => task.id)).toEqual([
    'title-global-settings',
    'task-auto-download',
    'task-receive-reward',
    'task-daily-sign',
    'title-daily',
  ]);

  const movedSign = state?.scriptTasks[scriptId].find((task) => task.id === 'task-daily-sign');
  const movedReward = state?.scriptTasks[scriptId].find((task) => task.id === 'task-receive-reward');
  expect(movedSign?.sectionId).toBe('title-global-settings');
  expect(movedReward?.sectionId).toBe('title-global-settings');
});

test('moves policy collections from context menu', async ({ page }) => {
  const scriptId = 'script-editor-policy-context-move';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略列表菜单脚本',
      description: '验证策略、策略组、策略集右键移动菜单',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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

    const makePolicy = (id: string, orderIndex: number, name: string): PolicyTable => ({
      id,
      scriptId: seedScript.id,
      orderIndex,
      data: {
        name,
        note: `${name}备注`,
        logPrint: null,
        curPos: 0,
        skipFlag: false,
        execCur: 0,
        execMax: 1,
        beforeAction: [],
        cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
        afterAction: [],
      },
    });
    const makeGroup = (id: string, orderIndex: number, name: string): PolicyGroupTable => ({
      id,
      scriptId: seedScript.id,
      orderIndex,
      data: {
        name,
        note: `${name}备注`,
      },
    });
    const makeSet = (id: string, orderIndex: number, name: string): PolicySetTable => ({
      id,
      scriptId: seedScript.id,
      orderIndex,
      data: {
        name,
        note: `${name}备注`,
      },
    });

    window.__AUTODAILY_MOCK__.reset();
    window.__AUTODAILY_MOCK__.seed({
      scripts: [seedScript],
      scriptTasks: {},
      policies: [
        makePolicy('policy-a', 0, '登录策略'),
        makePolicy('policy-b', 1, '领奖策略'),
        makePolicy('policy-c', 2, '商店策略'),
      ],
      policyGroups: [
        makeGroup('group-a', 0, '基础策略组'),
        makeGroup('group-b', 1, '扩展策略组'),
        makeGroup('group-c', 2, '收尾策略组'),
      ],
      policySets: [
        makeSet('set-a', 0, '主策略集'),
        makeSet('set-b', 1, '副策略集'),
        makeSet('set-c', 2, '回退策略集'),
      ],
      groupPolicies: {},
      setGroups: {},
    });
  }, script);
  await page.reload();

  await selectEditorMode(page, 'policy');
  await openCollectionContextMenu(page, 'editor-policy', 'policy-a');
  await page.getByTestId('editor-policy-move-current-bottom').evaluate((element: HTMLElement) => element.click());

  await selectEditorMode(page, 'policyGroup');
  await openCollectionContextMenu(page, 'editor-policy-group', 'group-c');
  await page.getByTestId('editor-policy-group-move-item').dispatchEvent('mouseenter');
  await page.getByTestId('editor-policy-group-move-item-group-a').dispatchEvent('mouseenter');
  await page.getByTestId('editor-policy-group-move-item-group-a-top').evaluate((element: HTMLElement) => element.click());

  await selectEditorMode(page, 'policySet');
  await openCollectionContextMenu(page, 'editor-policy-set', 'set-a');
  await page.getByTestId('editor-policy-set-move-item').dispatchEvent('mouseenter');
  await page.getByTestId('editor-policy-set-move-item-set-b').dispatchEvent('mouseenter');
  await page.getByTestId('editor-policy-set-move-item-set-b-bottom').evaluate((element: HTMLElement) => element.click());

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.policies.map((item) => item.id)).toEqual(['policy-b', 'policy-c', 'policy-a']);
  expect(state?.policyGroups.map((item) => item.id)).toEqual(['group-c', 'group-a', 'group-b']);
  expect(state?.policySets.map((item) => item.id)).toEqual(['set-b', 'set-a', 'set-c']);
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
        value: {
          type: 'string',
          value: '已完成',
        },
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

test('persists visionCountCompare as an if condition with nested branch steps', async ({ page }) => {
  const scriptId = 'script-editor-vision-count-condition';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '数量条件脚本',
      description: '验证判断数量大小改为 if 条件后仍可保存分支步骤',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'input-counter',
            key: 'input.counter',
            name: '计数器',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 3,
            description: 'Input · 整数',
          },
          {
            id: 'runtime-items',
            key: 'runtime.items',
            name: '结果集',
            namespace: 'runtime',
            valueType: 'list',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: 'Runtime · 列表',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-key-0').fill('ocrResults');
  await selectOptionByValue(page, 'editor-input-type-0', 'json');

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-if').click();

  await selectOptionByValue(page, 'editor-condition-type', 'visionCountCompare');
  await selectOptionByValue(page, 'editor-condition-vision-count-compare-input-var', 'input.ocrResults');
  await page.getByTestId('editor-condition-vision-count-compare-target-value').fill('领取');
  await page.getByTestId('editor-condition-vision-count-compare-expected-count').fill('2');
  await page.getByTestId('editor-branch-then').click();
  await page.getByTestId('editor-step-template-wait').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'if',
      con: {
        type: 'visionCountCompare',
        input_var: 'input.ocrResults',
        target_value: '领取',
        op: 'ge',
        expected_count: 2,
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

test('persists action sequence, vision rule, and task state forms', async ({ page }) => {
  const scriptId = 'script-editor-leaf-forms';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '叶子表单脚本',
      description: '验证动作序列、vision、taskControl 表单保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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
  await page.getByTestId('editor-branch-sequence').click();
  await expect(page.getByTestId('editor-step-template-click-point')).toBeVisible();
  await expect(page.getByTestId('editor-step-template-vision-search')).toHaveCount(0);
  await page.getByTestId('editor-step-template-click-point').click();
  await page.getByTestId('editor-step-template-wait').click();
  await page.getByRole('button', { name: '顶层步骤' }).click();

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
    steps: [
      {
        op: 'action',
        a: {
          ac: 'click',
          mode: 'point',
        },
      },
      {
        op: 'flowControl',
        a: {
          type: 'waitMs',
          ms: 1000,
        },
      },
    ],
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
      val: {
        type: 'float',
        value: 1.5,
      },
      expr: null,
    },
  });
  expect(task.data.steps[4]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'getVar',
      name: 'input.retryCount',
      default_val: {
        type: 'int',
        value: 3,
      },
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

test('switches preset and binding editors for launch click and wait steps', async ({ page }) => {
  const scriptId = 'script-editor-preset-binding';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '预设绑定脚本',
      description: '验证启动应用、点击和等待步骤的预设/绑定变量切换',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'var-package-name',
            key: 'input.packageName',
            name: '包名变量',
            namespace: 'input',
            valueType: 'string',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 'com.example.app',
            description: '',
          },
          {
            id: 'var-activity-name',
            key: 'input.activityName',
            name: 'Activity 变量',
            namespace: 'input',
            valueType: 'string',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: '.MainActivity',
            description: '',
          },
          {
            id: 'var-tap-point',
            key: 'input.tapPoint',
            name: '点击坐标变量',
            namespace: 'input',
            valueType: 'json',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: { x: 320, y: 480 },
            description: '',
          },
          {
            id: 'var-tap-percent',
            key: 'input.tapPercent',
            name: '点击百分比变量',
            namespace: 'input',
            valueType: 'json',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: { x: 0.5, y: 0.5 },
            description: '',
          },
          {
            id: 'var-target-text',
            key: 'input.targetText',
            name: '目标文字变量',
            namespace: 'input',
            valueType: 'string',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: '领取',
            description: '',
          },
          {
            id: 'var-target-label',
            key: 'input.targetLabel',
            name: '目标标签变量',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 1,
            description: '',
          },
          {
            id: 'var-wait-ms',
            key: 'input.waitMs',
            name: '等待毫秒变量',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 3000,
            description: '',
          },
          {
            id: 'var-search-hits',
            key: 'runtime.searchHits',
            name: 'OCR 结果',
            namespace: 'runtime',
            valueType: 'json',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: false,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: '',
          },
          {
            id: 'var-det-results',
            key: 'runtime.detResults',
            name: '检测结果',
            namespace: 'runtime',
            valueType: 'json',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: false,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: '',
          },
          {
            id: 'var-ocr-results',
            key: 'runtime.ocrResults',
            name: 'OCR 倒计时结果',
            namespace: 'runtime',
            valueType: 'json',
            ownerTaskId: 'preset_binding_task',
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: false,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: '',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);
  await page.evaluate((currentScriptId) => {
    const tasks: ScriptTaskTable[] = [
      {
        id: 'preset_binding_task',
        scriptId: currentScriptId,
        name: '预设绑定任务',
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
              label: '启动应用',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'launchApp',
                pkg_name: 'com.fixed.app',
                activity_name: '.FixedActivity',
              },
            },
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '点击坐标',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'click',
                mode: 'point',
                offset_x: 0,
                offset_y: 0,
                p: { x: 640, y: 360 },
              },
            },
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '点击百分比',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'click',
                mode: 'percent',
                offset_x: 0,
                offset_y: 0,
                p: { x: 0.5, y: 0.5 },
              },
            },
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '点击文字',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'action',
              a: {
                ac: 'click',
                mode: 'txt',
                input_var: 'runtime.searchHits',
                txt: '开始',
                enable_filter: true,
              },
            },
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
                input_var: 'runtime.detResults',
                idx: 0,
                enable_filter: true,
              },
            },
            {
              id: null,
              source_id: null,
              target_id: null,
              label: '等待',
              skip_flag: false,
              exec_cur: 0,
              exec_max: 1,
              op: 'flowControl',
              a: {
                type: 'waitMs',
                ms: 1000,
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
  await selectOptionByValue(page, 'editor-action-launch-pkg-source', 'expr');
  await expect(page.getByTestId('editor-action-launch-pkg-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-launch-pkg-var', 'input.packageName');
  await selectOptionByValue(page, 'editor-action-launch-activity-source', 'expr');
  await expect(page.getByTestId('editor-action-launch-activity-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-launch-activity-var', 'input.activityName');

  await page.getByTestId('editor-step-card-1').click();
  await selectOptionByValue(page, 'editor-action-click-point-source', 'expr');
  await expect(page.getByTestId('editor-action-click-point-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-click-point-var', 'input.tapPoint');

  await page.getByTestId('editor-step-card-2').click();
  await selectOptionByValue(page, 'editor-action-click-point-source', 'expr');
  await expect(page.getByTestId('editor-action-click-point-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-click-point-var', 'input.tapPercent');

  await page.getByTestId('editor-step-card-3').click();
  await selectOptionByValue(page, 'editor-action-click-text-filter-source', 'expr');
  await expect(page.getByTestId('editor-action-click-text-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-click-text-var', 'input.targetText');

  await page.getByTestId('editor-step-card-4').click();
  await selectOptionByValue(page, 'editor-action-click-label-filter-source', 'expr');
  await expect(page.getByTestId('editor-action-click-label-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-action-click-label-var', 'input.targetLabel');

  await page.getByTestId('editor-step-card-5').click();
  await selectOptionByValue(page, 'editor-flow-wait-binding-mode', 'expr');
  await expect(page.getByTestId('editor-flow-wait-runtime-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-flow-wait-variable-mode', 'input');
  await expect(page.getByTestId('editor-flow-wait-input-var')).toBeVisible();
  await selectOptionByValue(page, 'editor-flow-wait-input-var', 'input.waitMs');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];

  expect(task.data.steps[0]).toMatchObject({
    op: 'action',
    a: {
      ac: 'launchApp',
      pkg_name_expr: 'input.packageName',
      activity_name_expr: 'input.activityName',
    },
  });
  expect(task.data.steps[1]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'point',
      p_expr: 'input.tapPoint',
    },
  });
  expect(task.data.steps[2]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'percent',
      p_expr: 'input.tapPercent',
    },
  });
  expect(task.data.steps[3]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'txt',
      txt_expr: 'input.targetText',
    },
  });
  expect(task.data.steps[4]).toMatchObject({
    op: 'action',
    a: {
      ac: 'click',
      mode: 'labelIdx',
      idx_expr: 'input.targetLabel',
    },
  });
  expect(task.data.steps[5]).toMatchObject({
    op: 'flowControl',
    a: {
      type: 'waitMs',
      input_var: 'input.waitMs',
    },
  });
});

test('persists rhai code step from editor panel', async ({ page }) => {
  const scriptId = 'script-editor-rhai-step';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: 'Rhai 步骤脚本',
      description: '验证 Rhai 代码步骤保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
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
  await page.getByTestId('editor-step-template-rhai').click();
  await page.getByTestId('editor-step-card-0').click();
  await fillCodeEditor(page, 'editor-rhai-code', 'runtime.counter = (runtime.counter ?? 0) + 1;\nruntime.counter');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'rhai',
      code: 'runtime.counter = (runtime.counter ?? 0) + 1;\nruntime.counter',
      out_var: null,
    },
  });
});

test('persists clearVars selection from variable list', async ({ page }) => {
  const scriptId = 'script-editor-clear-vars';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '清空变量脚本',
      description: '验证 clearVars 从变量列表选择并保存',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'input-counter',
            key: 'input.counter',
            name: '计数器',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 3,
            description: 'Input · 整数',
          },
          {
            id: 'runtime-items',
            key: 'runtime.items',
            name: '结果集',
            namespace: 'runtime',
            valueType: 'list',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: 'Runtime · 列表',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-clear-vars').click();
  await page.getByTestId('editor-step-card-0').click();
  await page.getByTestId('editor-clear-vars-option-input.counter').click();
  await page.getByTestId('editor-clear-vars-option-runtime.items').click();

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];

  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'clearVars',
      names: ['input.counter', 'runtime.items'],
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
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'rename-me',
            key: 'input.oldVar',
            name: 'oldVar',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 0,
            description: '',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();
  await selectOptionByValue(page, 'editor-set-var-name', 'input.oldVar');
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
      val: {
        type: 'int',
        value: 0,
      },
      expr: null,
    },
  });
});

test('persists structured json value for setVar', async ({ page }) => {
  const scriptId = 'script-editor-setvar-json';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: 'JSON 变量脚本',
      description: '验证 setVar 对对象变量保存结构化 JSON',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'runtime-payload',
            key: 'runtime.payload',
            name: '负载',
            namespace: 'runtime',
            valueType: 'object',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: false,
            uiBindable: false,
            defaultValue: null,
            description: 'Runtime · 对象',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-0').click();
  await selectOptionByValue(page, 'editor-set-var-name', 'runtime.payload');
  await fillCodeEditor(page, 'editor-set-var-json', '{\n  "enabled": true,\n  "items": [1, 2]\n}');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  const [task] = state!.scriptTasks[scriptId];
  expect(task.data.steps[0]).toMatchObject({
    op: 'dataHanding',
    a: {
      type: 'setVar',
      name: 'runtime.payload',
      val: null,
      json_val: {
        enabled: true,
        items: [1, 2],
      },
      expr: null,
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

test('allows clearing policy name without forcing default text back', async ({ page }) => {
  const scriptId = 'script-editor-policy-empty-name';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略空名脚本',
      description: '验证策略名称清空后不会被默认值回填',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: null,
      txtDetModel: null,
      txtRecModel: null,
      createTime: '2026-03-26T08:00:00.000Z',
      updateTime: '2026-03-26T08:00:00.000Z',
      verName: '1.0.0',
      verNum: 1,
      latestVer: 1,
      downloadCount: 0,
      scriptType: 'dev',
      isValid: true,
      allowClone: true,
      variableCatalog: {
        version: 1,
        variables: [
          {
            id: 'rename-me',
            key: 'input.oldVar',
            name: 'oldVar',
            namespace: 'input',
            valueType: 'int',
            ownerTaskId: null,
            sourceType: 'manual',
            sourceStepId: null,
            readable: true,
            writable: true,
            persisted: true,
            uiBindable: true,
            defaultValue: 0,
            description: '',
          },
        ],
      },
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await selectEditorMode(page, 'policy');
  await page.getByTestId('editor-policy-create').click();
  await expect(page.getByTestId('editor-policy-name')).toHaveValue('策略 1');

  await page.getByTestId('editor-policy-name').fill('');
  await expect(page.getByTestId('editor-policy-name')).toHaveValue('');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.policies[0]?.data.name).toBe('');
});

test('loads img-det labels for policy condition label rules and saves idx', async ({ page }) => {
  const scriptId = 'script-editor-policy-label-condition';
  const script: StoredScriptTable = {
    id: scriptId,
    data: {
      name: '策略标签条件脚本',
      description: '验证策略命中条件里的标签召回规则会显示 index: labelName。',
      userId: 'tester',
      userName: 'Tester',
      runtimeType: 'rhai',
      sponsorshipQr: null,
      sponsorshipUrl: null,
      contactInfo: null,
      imgDetModel: {
        Yolo11: {
          baseModel: {
            intraThreadNum: 4,
            intraSpinning: true,
            interThreadNum: 1,
            interSpinning: true,
            executionProvider: 'CPU',
            inputWidth: 640,
            inputHeight: 640,
            modelSource: 'Custom',
            modelPath: 'D:\\models\\img-det.onnx',
            modelType: 'Yolo11',
          },
          classCount: 4,
          confidenceThresh: 0.25,
          iouThresh: 0.45,
          labelPath: 'D:\\models\\img-det.labels.yaml',
          txtIdx: 0,
          postprocessKind: 'LegacyNms',
        },
      },
      txtDetModel: null,
      txtRecModel: null,
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
  await page.getByTestId('editor-policy-name').fill('标签命中策略');

  await page.getByTestId('editor-policy-tab-condition').click();
  await page.getByRole('button', { name: '标签', exact: true }).click();
  await expect(page.getByTestId('editor-policy-condition-item-0-det-label-idx')).toContainText('0: 文本');
  await selectOptionByLabel(page, 'editor-policy-condition-item-0-det-label-idx', '1: 按钮');
  await expect(page.getByTestId('editor-policy-condition-item-0-det-label-idx')).toContainText('1: 按钮');

  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.policies[0]).toMatchObject({
    data: {
      cond: {
        type: 'group',
        items: [
          {
            type: 'detLabel',
            idx: 1,
          },
        ],
      },
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

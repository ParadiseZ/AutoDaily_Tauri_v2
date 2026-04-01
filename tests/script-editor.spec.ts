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
  await page.getByTestId(testId).click();
  await page.getByTestId(`${testId}-option-${value}`).click();
};

const dragStepByHandle = async (page: Page, fromIndex: number, toIndex: number) => {
  const handle = page.getByTestId(`editor-step-drag-${fromIndex}`);
  const target = page.getByTestId(`editor-step-card-${toIndex}`);
  const handleBox = await handle.boundingBox();
  const targetBox = await target.boundingBox();

  if (!handleBox || !targetBox) {
    throw new Error('step drag target is not visible');
  }

  await page.mouse.move(handleBox.x + handleBox.width / 2, handleBox.y + handleBox.height / 2);
  await page.mouse.down();
  await page.mouse.move(targetBox.x + targetBox.width / 2, targetBox.y + targetBox.height / 2, { steps: 10 });
  await page.mouse.up();
};

const dragRelationByHandle = async (page: Page, fromIndex: number, toIndex: number) => {
  const handle = page.getByTestId(`editor-relation-drag-${fromIndex}`);
  const target = page.getByTestId(`editor-relation-assigned-${toIndex === 0 ? 'policy-a' : 'policy-b'}`);
  await handle.dragTo(target);
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
  await selectOptionByValue(page, 'editor-task-type', 'child');
  await page.getByTestId('editor-task-hidden').check();

  await page.getByTestId('editor-tab-inputs').click();
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-key-0').fill('activitySweepCount');
  await page.getByTestId('editor-input-value-0').fill('5');
  await page.getByTestId('editor-input-add').click();
  await page.getByTestId('editor-input-remove-1').click();

  await page.getByTestId('editor-tab-ui').click();
  await page.getByTestId('editor-ui-template-number').click();
  await page.getByTestId('editor-ui-field-label-0').fill('扫荡活动');
  await page.getByTestId('editor-ui-field-bind-0').click();
  await page.getByTestId('editor-ui-field-bind-0-menu').getByText('activitySweepCount').click();
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
  expect(task.taskType).toBe('child');
  expect(task.isHidden).toBe(true);
  expect(task.data.variables).toEqual({ activitySweepCount: 8 });
  expect(savedScript?.data.variableCatalog.variables).toHaveLength(1);
  expect(savedScript?.data.variableCatalog.variables[0]).toMatchObject({
    key: 'input.activitySweepCount',
    name: 'activitySweepCount',
    namespace: 'input',
    valueType: 'int',
    ownerTaskId: task.id,
    persisted: true,
    uiBindable: true,
    defaultValue: 8,
  });
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
  await page.getByLabel('输出变量').fill('runtime.hit');
  await page.getByRole('button', { name: '添加关键字' }).click();
  await page.getByTestId('editor-search-rule-item-0-keyword').fill('领取');

  await page.getByTestId('editor-step-template-set-task-state').click();
  await page.getByTestId('editor-step-card-2').click();
  await page.getByLabel('目标 ID').fill('daily_task');
  await page.getByLabel('状态值为真').uncheck();

  await page.getByTestId('editor-step-template-set-var').click();
  await page.getByTestId('editor-step-card-3').click();
  await page.getByTestId('editor-set-var-name').click();
  await page.getByTestId('editor-set-var-name-menu').getByText('sweepLimit').click();
  await page.getByTestId('editor-set-var-value').fill('1.5');

  await page.getByTestId('editor-step-template-get-var').click();
  await page.getByTestId('editor-step-card-4').click();
  await page.getByTestId('editor-get-var-name').click();
  await page.getByTestId('editor-get-var-name-menu').getByText('retryCount').click();
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
      out_var: 'runtime.hit',
      rule: {
        type: 'group',
        op: 'And',
        scope: 'Global',
        items: [
          {
            type: 'keyword',
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

  await page.getByTestId('editor-mode-policy').click();
  await page.getByTestId('editor-policy-create').click();
  await page.getByTestId('editor-policy-name').fill('领奖策略');

  await page.getByTestId('editor-policy-tab-condition').click();
  await page.getByRole('button', { name: '添加关键字' }).click();
  await page.getByTestId('editor-policy-condition-item-0-keyword').fill('领取');

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
            type: 'keyword',
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

  await page.getByTestId('editor-mode-policyGroup').click();
  await dragRelationByHandle(page, 1, 0);
  await expect(page.getByTestId('editor-relation-assigned-policy-b')).toContainText('1');
  await page.getByTestId('editor-save').click();

  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.groupPolicies['group-a']).toEqual(['policy-b', 'policy-a']);
});

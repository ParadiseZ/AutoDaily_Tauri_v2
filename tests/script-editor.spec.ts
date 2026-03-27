import { expect, test, type Page } from '@playwright/test';
import type { ScriptTable, ScriptTaskTable } from '../src/types/bindings';

type StoredScriptTable = Omit<ScriptTable, 'data'> & {
  data: Omit<ScriptTable['data'], 'downloadCount' | 'latestVer' | 'verNum'> & {
    downloadCount: number;
    latestVer: number;
    verNum: number;
  };
};

declare global {
  interface Window {
    __AUTODAILY_MOCK__?: {
      getState: () => {
        scripts: StoredScriptTable[];
        scriptTasks: Record<string, ScriptTaskTable[]>;
      };
      reset: () => unknown;
      seed: (partial: {
        scripts?: StoredScriptTable[];
        scriptTasks?: Record<string, ScriptTaskTable[]>;
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

  await page.getByTestId('editor-tab-ui').click();
  await page.getByTestId('editor-ui-template-number').click();
  await page.getByTestId('editor-ui-field-label-0').fill('扫荡活动');
  await selectOptionByValue(page, 'editor-ui-field-bind-0', 'activitySweepCount');

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
  expect(task.name).toBe('日常主流程');
  expect(task.taskType).toBe('child');
  expect(task.isHidden).toBe(true);
  expect(task.data.variables).toEqual({ activitySweepCount: 5 });
  expect(task.data.uiData).toEqual({
    layout: 'horizontal',
    fields: [
      {
        key: 'activitySweepCount',
        label: '扫荡活动',
        control: 'number',
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
      cloudId: null,
    },
  };

  await seedEditorState(page, script);

  await page.getByTestId('editor-tab-steps').click();
  await page.getByTestId('editor-step-template-if').click();

  await selectOptionByValue(page, 'editor-condition-type', 'varCompare');
  await page.getByLabel('变量名').fill('runtime.ocr_text');
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
        var_name: 'runtime.ocr_text',
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

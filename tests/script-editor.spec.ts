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
  await page.getByTestId('editor-step-card-1').dragTo(page.getByTestId('editor-step-card-0'));

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

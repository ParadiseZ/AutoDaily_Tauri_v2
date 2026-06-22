import { expect, test, type Page } from '@playwright/test';
import type { ScriptTable } from '../src/types/bindings';

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
      };
      reset: () => unknown;
      seed: (partial: Record<string, unknown>) => unknown;
    };
  }
}

const openCleanScriptsPage = async (page: Page) => {
  await page.goto('/scripts');
  await page.evaluate(() => {
    if (!window.__AUTODAILY_MOCK__) {
      throw new Error('browser mock backend is not available');
    }

    window.__AUTODAILY_MOCK__.reset();
  });
  await page.reload();
  await expect(page.getByRole('heading', { name: '本地列表' })).toBeVisible();
};

const selectOptionByValue = async (page: Page, testId: string, value: string) => {
  await page.getByTestId(testId).click();
  await page.getByTestId(`${testId}-option-${value}`).click();
};

const openModelTab = async (page: Page, tabId: 'imgDet' | 'txtDet' | 'txtRec') => {
  await page.getByTestId(`script-models-tab-${tabId}`).click();
};

const getStoredScript = async (page: Page) => {
  const state = await page.evaluate(() => window.__AUTODAILY_MOCK__?.getState());
  expect(state?.scripts).toHaveLength(1);
  return state!.scripts[0];
};

const seedPublishedScript = async (
  page: Page,
  options?: { allowClone?: boolean; authSeed?: boolean; authUserId?: string },
) => {
  const scriptId = `published-${Date.now()}`;
  await page.evaluate(({ scriptId, emptyVariableCatalog, allowClone, authSeed, authUserId }) => {
    window.__AUTODAILY_MOCK__?.seed({
      authSession: authSeed
        ? {
            accessToken: 'mock-access-token',
            refreshToken: 'mock-refresh-token',
            username: 'tester',
            message: null,
          }
        : null,
      userProfile: authSeed
        ? {
            id: authUserId,
            username: 'tester',
            email: 'tester@example.com',
            isDeveloper: false,
            lastScriptUploadTime: '',
            lastUsernameChangeTime: '',
            sponsorUntil: null,
            authStage: 1,
          }
        : null,
      scripts: [
        {
          id: scriptId,
          data: {
            name: '云端脚本样例',
            description: '用于验证本地列表上的云端脚本操作限制。',
            contentMd: '# 2026-05-12\n\n**v1.0.0 更新日志**\n\n- 已发布。',
            userId: 'local-user',
            userName: 'CloudAuthor',
            runtimeType: 'rhai',
            platform: 'android',
            sponsorshipQr: null,
            sponsorshipUrl: null,
            contactInfo: null,
            imgDetModel: null,
            txtDetModel: null,
            txtRecModel: null,
            createTime: '2026-05-12T08:00:00.000Z',
            updateTime: '2026-05-12T08:00:00.000Z',
            verName: '1.0.0',
            verNum: 1,
            latestVer: 1,
            downloadCount: 9,
            scriptType: 'published',
            isValid: true,
            allowClone,
            minAppVersion: '0.1.0',
            minRuntimeSchema: 1,
            requiredFeatures: ['onnxInference', 'runtime:rhai', 'device:android'],
            variableCatalog: emptyVariableCatalog,
            cloudId: 'cloud-script-1',
            runtimeSettings: {
              recoveryTaskId: null,
              clickRandomOffset: 0,
            },
          },
        },
      ],
    });
  }, {
    scriptId,
    emptyVariableCatalog,
    allowClone: options?.allowClone ?? true,
    authSeed: options?.authSeed ?? false,
    authUserId: options?.authUserId ?? 'local-user',
  });
  await page.reload();
  await expect(page.getByRole('heading', { name: '云端脚本样例' })).toBeVisible();
};

test.beforeEach(async ({ page }) => {
  await openCleanScriptsPage(page);
});

test('disables submit when script name is blank', async ({ page }) => {
  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  const nameInput = dialog.getByTestId('script-basic-name');
  const submitButton = dialog.getByTestId('script-submit');

  await expect(nameInput).toHaveValue('未命名脚本 1');
  await nameInput.fill('');

  await expect(submitButton).toBeDisabled();
});

test('aligns YOLO model fields with consistent spacing', async ({ page }) => {
  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await dialog.getByTestId('script-dialog-tab-models').click();
  await openModelTab(page, 'imgDet');
  await selectOptionByValue(page, 'script-models-img-det-kind', 'Yolo11');

  const fields = [
    dialog.getByTestId('script-models-img-det-kind'),
    dialog.getByTestId('script-models-img-det-base-model-source'),
    dialog.getByTestId('script-models-img-det-base-model-path'),
    dialog.getByTestId('script-models-img-det-label-path'),
    dialog.getByTestId('script-models-img-det-class-count'),
    dialog.getByTestId('script-models-img-det-base-input-width'),
  ];
  const boxes = await Promise.all(fields.map((field) => field.boundingBox()));

  for (const box of boxes) {
    expect(box).not.toBeNull();
    expect(box!.x).toBeCloseTo(boxes[0]!.x, 0);
  }

  const verticalFields = boxes.slice(2);
  for (let index = 1; index < verticalFields.length; index += 1) {
    const previous = verticalFields[index - 1]!;
    const current = verticalFields[index]!;
    expect(current.y - (previous.y + previous.height)).toBeCloseTo(16, 0);
  }
});

test('creates a local script with basic, model, and sponsorship settings', async ({ page }) => {
  const scriptName = `每日清体力 ${Date.now()}`;
  const description = '覆盖基本信息、模型信息和赞助信息的创建流程';
  const qrDataUrl =
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2P8//8/AwAI/AL+X2HFVQAAAABJRU5ErkJggg==';

  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await expect(dialog).toBeVisible();

  await dialog.getByTestId('script-basic-name').fill(scriptName);
  await dialog.getByTestId('script-basic-description').fill(description);
  await selectOptionByValue(page, 'script-basic-runtime-type', 'javaScript');
  await dialog.getByTestId('script-basic-version-name').fill('2.0.0');
  await dialog.getByTestId('script-basic-version-num').fill('7');
  await dialog.getByTestId('script-basic-allow-clone').uncheck();

  await dialog.getByTestId('script-dialog-tab-models').click();

  await openModelTab(page, 'imgDet');
  await selectOptionByValue(page, 'script-models-img-det-kind', 'Yolo11');
  await selectOptionByValue(page, 'script-models-img-det-base-model-source', 'Custom');
  await selectOptionByValue(page, 'script-models-img-det-base-execution-provider', 'DirectML');
  await dialog.getByTestId('script-models-img-det-base-model-path').fill('D:\\models\\img-det.onnx');
  await dialog.getByTestId('script-models-img-det-label-path').fill('D:\\models\\img-det.labels.yaml');
  await expect(dialog.getByTestId('script-models-img-det-class-count')).toHaveValue('4');
  await dialog.getByTestId('script-models-img-det-confidence').fill('0.55');
  await dialog.getByTestId('script-models-img-det-iou').fill('0.35');

  await openModelTab(page, 'txtDet');
  await selectOptionByValue(page, 'script-models-txt-det-kind', 'PaddleDbNet');
  await selectOptionByValue(page, 'script-models-txt-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-det-base-model-path').fill('D:\\models\\txt-det.onnx');
  await dialog.getByTestId('script-models-txt-det-db-thresh').fill('0.42');
  await dialog.getByTestId('script-models-txt-det-db-box-thresh').fill('0.66');
  await dialog.getByTestId('script-models-txt-det-unclip-ratio').fill('2.4');
  await dialog.getByTestId('script-models-txt-det-use-dilation').check();

  await openModelTab(page, 'txtRec');
  await selectOptionByValue(page, 'script-models-txt-rec-kind', 'PaddleCrnn');
  await selectOptionByValue(page, 'script-models-txt-rec-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-rec-base-model-path').fill('D:\\models\\txt-rec.onnx');
  await dialog.getByTestId('script-models-txt-rec-dict-path').fill('D:\\models\\keys.txt');
  await selectOptionByValue(page, 'script-models-txt-rec-resize-filter', 'Triangle');
  await selectOptionByValue(page, 'script-models-txt-rec-processing-mode', 'MicroBatch');
  await dialog.getByTestId('script-models-txt-rec-micro-batch-size').fill('3');
  await dialog.getByTestId('script-models-txt-rec-width-bucket-step').fill('24');

  await dialog.getByTestId('script-dialog-tab-support').click();
  await dialog.getByTestId('script-support-contact-info').fill('tester@example.com');
  await dialog.getByTestId('script-support-sponsorship-url').fill('https://example.com/sponsor');
  await dialog.getByTestId('script-support-sponsorship-qr-input').fill(qrDataUrl);
  await expect(dialog.getByTestId('script-support-sponsorship-qr-source')).toContainText('Data URL');

  await dialog.getByTestId('script-submit').click();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole('heading', { name: scriptName })).toBeVisible();

  const script = await getStoredScript(page);
  expect(script.data.name).toBe(scriptName);
  expect(script.data.description).toBe(description);
  expect(script.data.runtimeType).toBe('javaScript');
  expect(script.data.verName).toBe('2.0.0');
  expect(script.data.verNum).toBe(7);
  expect(script.data.allowClone).toBe(false);
  expect(script.data.variableCatalog).toEqual(emptyVariableCatalog);

  expect(script.data.imgDetModel).toEqual({
    Yolo11: expect.objectContaining({
      classCount: 4,
      confidenceThresh: 0.55,
      iouThresh: 0.35,
      labelPath: 'D:\\models\\img-det.labels.yaml',
      postprocessKind: 'LegacyNms',
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        executionProvider: 'DirectML',
        modelPath: 'D:\\models\\img-det.onnx',
      }),
    }),
  });

  expect(script.data.txtDetModel).toEqual({
    PaddleDbNet: expect.objectContaining({
      dbThresh: 0.42,
      dbBoxThresh: 0.66,
      unclipRatio: 2.4,
      useDilation: true,
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        modelPath: 'D:\\models\\txt-det.onnx',
      }),
    }),
  });

  expect(script.data.txtRecModel).toEqual({
    PaddleCrnn: expect.objectContaining({
      dictPath: 'D:\\models\\keys.txt',
      resizeFilter: 'Triangle',
      processingMode: 'MicroBatch',
      microBatchSize: 3,
      widthBucketStep: 24,
      baseModel: expect.objectContaining({
        modelSource: 'Custom',
        modelPath: 'D:\\models\\txt-rec.onnx',
      }),
    }),
  });

  expect(script.data.contactInfo).toBe('tester@example.com');
  expect(script.data.sponsorshipUrl).toBe('https://example.com/sponsor');
  expect(script.data.sponsorshipQr).toBe(qrDataUrl);

  await page.reload();
  await expect(page.getByRole('heading', { name: scriptName })).toBeVisible();
});

test('creates a local script with yolo26 detector settings', async ({ page }) => {
  const scriptName = `YOLO26 检测 ${Date.now()}`;

  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await dialog.getByTestId('script-basic-name').fill(scriptName);
  await dialog.getByTestId('script-basic-description').fill('用于验证 YOLO26 检测模型配置。');
  await dialog.getByTestId('script-dialog-tab-models').click();

  await openModelTab(page, 'imgDet');
  await selectOptionByValue(page, 'script-models-img-det-kind', 'Yolo26');
  await selectOptionByValue(page, 'script-models-img-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-img-det-base-model-path').fill('D:\\models\\img-det-yolo26.onnx');
  await dialog.getByTestId('script-models-img-det-label-path').fill('D:\\models\\img-det-yolo26.labels.yaml');
  await expect(dialog.getByTestId('script-models-img-det-class-count')).toHaveValue('4');
  await expect(dialog.getByTestId('script-models-img-det-confidence')).toHaveCount(0);
  await expect(dialog.getByTestId('script-models-img-det-iou')).toHaveCount(0);

  await openModelTab(page, 'txtDet');
  await selectOptionByValue(page, 'script-models-txt-det-kind', 'Yolo26');
  await selectOptionByValue(page, 'script-models-txt-det-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-det-base-model-path').fill('D:\\models\\txt-det-yolo26.onnx');
  await dialog.getByTestId('script-models-txt-det-label-path').fill('D:\\models\\txt-det-yolo26.labels.yaml');
  await expect(dialog.getByTestId('script-models-txt-det-class-count')).toHaveValue('4');
  await expect(dialog.getByTestId('script-models-txt-det-confidence')).toHaveCount(0);
  await expect(dialog.getByTestId('script-models-txt-det-iou')).toHaveCount(0);

  await dialog.getByTestId('script-submit').click();
  await expect(dialog).not.toBeVisible();

  const script = await getStoredScript(page);
  expect(script.data.imgDetModel).toEqual({
    Yolo26: expect.objectContaining({
      classCount: 4,
      confidenceThresh: null,
      iouThresh: null,
      labelPath: 'D:\\models\\img-det-yolo26.labels.yaml',
      postprocessKind: 'EndToEnd',
      baseModel: expect.objectContaining({
        modelPath: 'D:\\models\\img-det-yolo26.onnx',
        modelType: 'Yolo26',
      }),
    }),
  });

  expect(script.data.txtDetModel).toEqual({
    Yolo26: expect.objectContaining({
      classCount: 4,
      confidenceThresh: null,
      iouThresh: null,
      labelPath: 'D:\\models\\txt-det-yolo26.labels.yaml',
      postprocessKind: 'EndToEnd',
      txtIdx: 0,
      baseModel: expect.objectContaining({
        modelPath: 'D:\\models\\txt-det-yolo26.onnx',
        modelType: 'Yolo26',
      }),
    }),
  });
});

test('hides built-in crnn dict path and clears custom dict when switching back', async ({ page }) => {
  const scriptName = `CRNN 内置字典 ${Date.now()}`;

  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await dialog.getByTestId('script-basic-name').fill(scriptName);
  await dialog.getByTestId('script-basic-description').fill('验证 CRNN 内置字典路径隐藏与清理。');
  await dialog.getByTestId('script-dialog-tab-models').click();

  await openModelTab(page, 'txtRec');
  await selectOptionByValue(page, 'script-models-txt-rec-kind', 'PaddleCrnn');
  await expect(dialog.getByTestId('script-models-txt-rec-dict-path')).toHaveCount(0);

  await selectOptionByValue(page, 'script-models-txt-rec-base-model-source', 'Custom');
  await dialog.getByTestId('script-models-txt-rec-base-model-path').fill('D:\\models\\txt-rec.onnx');
  await dialog.getByTestId('script-models-txt-rec-dict-path').fill('D:\\models\\custom-dict.txt');

  await selectOptionByValue(page, 'script-models-txt-rec-base-model-source', 'BuiltIn');
  await expect(dialog.getByTestId('script-models-txt-rec-dict-path')).toHaveCount(0);

  await dialog.getByTestId('script-submit').click();
  await expect(dialog).not.toBeVisible();

  const script = await getStoredScript(page);
  expect(script.data.txtRecModel).toEqual({
    PaddleCrnn: expect.objectContaining({
      dictPath: null,
      baseModel: expect.objectContaining({
        modelSource: 'BuiltIn',
      }),
    }),
  });
});

test('shows inline validation summary instead of stacking a second dialog on create', async ({ page }) => {
  await page.getByTestId('script-list-create-button').click();

  const dialog = page.getByRole('dialog', { name: '新建脚本' });
  await dialog.getByTestId('script-basic-description').fill('');
  await dialog.getByTestId('script-basic-version-name').fill('');
  await dialog.getByTestId('script-submit').click();

  await expect(dialog).toBeVisible();
  await expect(page.getByRole('dialog')).toHaveCount(1);
  await expect(dialog.getByTestId('script-info-validation-summary')).toContainText('描述');
  await expect(dialog.getByTestId('script-info-validation-summary')).toContainText('版本名称');
});

test('hides edit and upload actions for published scripts in local list', async ({ page }) => {
  await seedPublishedScript(page);

  await expect(page.getByRole('button', { name: '打开编辑器' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '编辑信息' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '上传' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '克隆为本地脚本' })).toBeVisible();
  await expect(page.getByText('云端脚本需先克隆为本地脚本后，才能编辑或再次上传。')).toBeVisible();
});

test('hides clone action when published script does not allow cloning', async ({ page }) => {
  await seedPublishedScript(page, { allowClone: false, authSeed: true, authUserId: 'another-user' });

  await expect(page.getByRole('button', { name: '克隆为本地脚本' })).toHaveCount(0);
});

test('script market prompts login when opened unauthenticated', async ({ page }) => {
  await page.goto('/market');

  await expect(page.getByRole('heading', { name: '脚本市场', exact: true })).toBeVisible();
  await expect(page.getByText(/登录后/)).toBeVisible();
  const authDialog = page.getByRole('dialog', { name: '欢迎回来' });
  await expect(authDialog).toBeVisible();
  await expect(authDialog.getByRole('button', { name: '登录' }).first()).toBeVisible();
});

test('script market refresh does not reopen login modal when already authenticated', async ({ page }) => {
  await page.goto('/market');
  await page.evaluate(() => {
    window.__AUTODAILY_MOCK__?.reset();
    window.__AUTODAILY_MOCK__?.seed({
      authSession: {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        username: 'tester',
        message: null,
      },
      userProfile: {
        id: 'local-user',
        username: 'tester',
        email: 'tester@example.com',
        isDeveloper: false,
        lastScriptUploadTime: '',
        lastUsernameChangeTime: '',
        sponsorUntil: null,
        authStage: 1,
      },
    });
  });

  await page.reload();
  await expect(page.getByRole('heading', { name: '脚本市场', exact: true })).toBeVisible();
  await expect(page.getByRole('dialog', { name: '欢迎回来' })).toHaveCount(0);
});

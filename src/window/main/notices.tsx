import { Notice as NoticeType } from '@app/bindings/Notice';
import Notice from '../../components/Notice.tsx';
import { useState, useEffect } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export default function NoticesList() {
  const [notices, setNotices] = useState<NoticeType[]>([]);
  useEffect(() => {
    invoke<NoticeType[]>('get_all_notices').then((fetchedNotices: NoticeType[]) => {
      setNotices(fetchedNotices);
    });
    const listener = listen('vrcmrd:notice', (event) => {
      const newNotice = event.payload as NoticeType;
      setNotices((prevNotices) => [newNotice, ...prevNotices]);
    });
    const reloadListener = () => {
      invoke<NoticeType[]>('get_all_notices').then((fetchedNotices: NoticeType[]) => {
        setNotices(fetchedNotices);
      });
    };
    document.addEventListener('vrcmrd:soft-reload', reloadListener);
    return () => {
      listener.then((unlisten) => unlisten());
      document.removeEventListener('vrcmrd:soft-reload', reloadListener);
    };
  }, []);
  return (<div class="flex-1 min-h-0 min-w-0 w-full overflow-hidden bg-gray-100 dark:bg-gray-900 overflow-y-auto overflow-x-hidden">
    <div class="flex flex-col space-y-4 h-full w-full min-h-0 min-w-0">
      {notices.map((notice) => (
        <Notice key={notice.createdAt} notice={notice} />
      ))}
      <Notice notice={{
        title: "Welcome to VRCMRD!",
        message: "This is a sample notice to demonstrate the notice system. You can manage advisories and view important information here.",
        level: 4 as any,
        createdAt: new Date().toISOString(),
        local: true,
        sendNotification: false,
        sendTts: false,
        relevantGroupId: null,
        relevantAdvisoryId: null,
        relevantUserId: null,
      }} />
      <Notice notice={{
        title: "Welcome to VRCMRD!",
        message: "This is a sample notice to demonstrate the notice system. You can manage advisories and view important information here.",
        level: 3 as any,
        createdAt: new Date().toISOString(),
        local: true,
        sendNotification: false,
        sendTts: false,
        relevantGroupId: null,
        relevantAdvisoryId: null,
        relevantUserId: null,
      }} />
      <Notice notice={{
        title: "Welcome to VRCMRD!",
        message: "This is a sample notice to demonstrate the notice system. You can manage advisories and view important information here.",
        level: 2 as any,
        createdAt: new Date().toISOString(),
        local: true,
        sendNotification: false,
        sendTts: false,
        relevantGroupId: null,
        relevantAdvisoryId: null,
        relevantUserId: null,
      }} />
      <Notice notice={{
        title: "Welcome to VRCMRD!",
        message: "This is a sample notice to demonstrate the notice system. You can manage advisories and view important information here.",
        level: 1 as any,
        createdAt: new Date().toISOString(),
        local: true,
        sendNotification: false,
        sendTts: false,
        relevantGroupId: null,
        relevantAdvisoryId: null,
        relevantUserId: null,
      }} />
      <Notice notice={{
        title: "Welcome to VRCMRD!",
        message: "This is a sample notice to demonstrate the notice system. You can manage advisories and view important information here.",
        level: 0 as any,
        createdAt: new Date().toISOString(),
        local: true,
        sendNotification: false,
        sendTts: false,
        relevantGroupId: null,
        relevantAdvisoryId: null,
        relevantUserId: null,
      }} />
    </div>
  </div>);
}
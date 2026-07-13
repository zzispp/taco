export type NoticeType = '1' | '2';
export type NoticeStatus = '0' | '1';

export type Notice = Readonly<{
  notice_id: string;
  notice_title: string;
  notice_type: NoticeType;
  notice_content: string;
  status: NoticeStatus;
  create_by: string;
  create_time: string;
  update_by: string | null;
  update_time: string | null;
  remark: string | null;
}>;

export type NoticeSummary = Readonly<
  Pick<
    Notice,
    'notice_id' | 'notice_title' | 'notice_type' | 'status' | 'create_by' | 'create_time'
  >
>;

export type NoticeInput = Readonly<{
  notice_title: string;
  notice_type: NoticeType;
  notice_content: string;
  status: NoticeStatus;
  remark: string | null;
}>;

export type NoticeFilters = Readonly<{
  notice_title?: string;
  create_by?: string;
  notice_type?: NoticeType | '';
}>;

export type NoticeTopItem = Readonly<{
  notice_id: string;
  notice_title: string;
  notice_type: NoticeType;
  create_by: string;
  create_time: string;
  is_read: boolean;
}>;

export type NoticeTopResponse = Readonly<{
  items: NoticeTopItem[];
  unread_count: number;
}>;

export type NoticeReader = Readonly<{
  user_id: string;
  user_name: string;
  nick_name: string;
  dept_name: string | null;
  phonenumber: string | null;
  read_time: string;
}>;

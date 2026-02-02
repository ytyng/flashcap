export interface Arrow {
  id: string;
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export interface ArrowSettings {
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export type MaskMode = "mosaic" | "blur" | "fill";

export interface MaskRect {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  mode: MaskMode;
  color: string;
}

export interface MaskSettings {
  mode: MaskMode;
  color: string;
  blurRadius: number;
  mosaicBlockSize: number;
}

export type ShapeType = "rect" | "ellipse";

export interface Shape {
  id: string;
  type: ShapeType;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export interface ShapeSettings {
  type: ShapeType;
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export interface TextAnnotation {
  id: string;
  x: number;
  y: number;
  text: string;
  fontSize: number;
  color: string;
  bold: boolean;
  italic: boolean;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export interface TextSettings {
  fontSize: number;
  color: string;
  bold: boolean;
  italic: boolean;
  whiteStroke: boolean;
  dropShadow: boolean;
}

//
//  FPLevelView.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import <Cocoa/Cocoa.h>
#import "NSOpenGLView+Helpers.h"
#import "FPGameProtocols.h"

typedef enum
{
	FPDragHandleNone = 0,
	
	FPDragHandleTopLeft,
	FPDragHandleBottomLeft,
	FPDragHandleTopRight,
	FPDragHandleBottomRight,
	
	FPDragHandleMiddleLeft,
	FPDragHandleMiddleTop,
	FPDragHandleMiddleRight,
	FPDragHandleMiddleBottom,
	
	FPDragHandleCenter
}  FPDragHandle;

@protocol FPLevelViewDataSource

@property (readonly) NSMutableArray *gameObjects;
@property (readonly) NSMutableIndexSet *selectedIndices;
@property (readwrite, assign) id activeFactory;

- (void)addNewGameObject:(id<FPGameObject>)gameObject;
- (void)beginMove;
- (void)endMoveWithPoint:(CGPoint)point;
- (void)beginResizeWithHandle:(FPDragHandle)dragHandle;
- (void)endResizeWithPoint:(CGPoint)point;

@end

@interface FPLevelView : NSOpenGLView
{
	IBOutlet id<FPLevelViewDataSource> dataSource;
	CGPoint beginMovePoint;
	CGPoint endMovePoint;
	FPDragHandle currentHandle;
	
	BOOL drawingSelection;
	CGPoint beginSelection;
	CGPoint endSelection;
}

- (CGPoint)pointFromHandle:(FPDragHandle)handle aroundRect:(CGRect)rect;
- (BOOL)respondsGameObject:(id<FPGameObject>)gameObject toDragHandle:(FPDragHandle)dragHandle;
- (void)resizeDraggedObjectLeft:(float)x;
- (void)resizeDraggedObjectRight:(float)x;
- (void)resizeDraggedObjectTop:(float)y;
- (void)resizeDraggedObjectBottom:(float)y;
- (void)moveDraggedObjectWithX:(float)x y:(float)y;
- (void)moveSelectedObjectsWithX:(float)x y:(float)y;
- (BOOL)isWidthHandle:(FPDragHandle)handle;
- (BOOL)isHeightHandle:(FPDragHandle)handle;
- (void)drawGrid;
- (void)drawHandlesOnGameObject:(id<FPGameObject>)gameObject;
- (void)setDraggedObjectX:(float)x y:(float)y;
- (id<FPGameObject>)draggedObject;

@end
